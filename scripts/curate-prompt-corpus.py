"""Build a local preview corpus from saved public snapshots; never execute prompts.

Requires pyarrow for DiffusionDB. Raw files, output and rejection counts remain
in the explicitly supplied directory. No network or database access here.
"""
import argparse
import csv
import hashlib
import json
import math
import re
from collections import Counter
from pathlib import Path

RISK = re.compile(r"\b(nsfw|nude|nudity|naked|porn\w*|sex(?:ual|y)?|erotic|fetish|breasts?|nipples?|genitals?|gore|gory|rape|suicide|jailbreak|DAN|malware|ransomware|phishing|explosive|terroris\w*|lolita|loli|underage|topless|lingerie|bikini|swimsuit|seductive|sensual)\b|色情|裸体|越狱|绕过.{0,8}(安全|限制)|忽略.{0,8}(规则|限制)", re.I)
NOISE = re.compile(r"https?://|\b(?:subscribe|follow me|buy now|test prompt|lorem ipsum)\b|\x00", re.I)
ADULT = re.compile(r'\b(milf|boobs?|busty|cleavage|panties|thong|undress\w*|see.through|bare.skin|provocative|voluptuous|sensuous|lust\w*|naughty|bdsm|hentai|waifu|schoolgirl|teen\w*)\b|\?{4,}|\ufffd', re.I)
LIGHT = re.compile(r"\b(light(?:ing)?|sunlight|backlit|golden hour|shadow|illuminat\w*|soft light|neon)\b", re.I)
STYLE = re.compile(r"\b(photograph\w*|cinematic|watercolou?r|illustration|oil painting|render|3d|isometric|vector|pixel art|line art|sketch|minimalist|macro)\b", re.I)
LAYOUT = re.compile(r"\b(composition|perspective|wide angle|close up|closeup|depth of field|bokeh|symmetr\w*|background|foreground|aerial|panorama|portrait)\b", re.I)

def digest(text):
    return hashlib.sha256(text.encode()).hexdigest()

def normalized(text):
    return ' '.join(re.findall(r'\w+', text.casefold()))

def reject_text(text, minimum=80, maximum=16000):
    if not isinstance(text, str) or not minimum <= len(text.strip()) <= maximum:
        return 'length'
    if RISK.search(text) or ADULT.search(text):
        return 'risk_terms'
    if NOISE.search(text):
        return 'noise_or_links'
    words = normalized(text).split()
    if len(set(words)) < 15 and not re.search(r'[\u4e00-\u9fff]', text):
        return 'low_information'
    return None

def image_score(row):
    text = row.get('prompt')
    reason = reject_text(text, maximum=1600)
    if reason:
        return None, reason
    for key in ('image_nsfw', 'prompt_nsfw'):
        value = row.get(key)
        if not isinstance(value, (int, float)) or not math.isfinite(value) or not 0 <= value <= 0.05:
            return None, 'safety_score'
    if min(row.get('width') or 0, row.get('height') or 0) < 512 or not 20 <= (row.get('step') or 0) <= 100 or not 4 <= (row.get('cfg') or 0) <= 15:
        return None, 'generation_parameters'
    dimensions = sum(bool(p.search(text)) for p in (LIGHT, STYLE, LAYOUT))
    if dimensions < 2:
        return None, 'missing_visual_directions'
    words = normalized(text).split()
    phrases = [tuple(words[i:i + 8]) for i in range(max(0, len(words) - 7))]
    if len(words) > 150 or len(set(words)) / len(words) < 0.7 or len(set(phrases)) != len(phrases):
        return None, 'keyword_stuffing'
    if re.search(r'\b\d{4,}\s*(k|mm)\b|\bage\s+\d\s+\d', text, re.I):
        return None, 'malformed_parameters'
    return dimensions * 10 + min(len(set(words)), 60) / 10, None

def diffusion_category(text):
    # This older dataset uses "portrait" even for animals/art. Prefer the actual
    # visual medium; generic photography alone must not imply a human portrait.
    if re.search(r'\b(product|packaging|bottle|perfume|sneaker|cosmetic)\b', text, re.I):
        return 'cat-image-1'
    if re.search(r'\b(illustration|poster|vector|watercolou?r|painting|anime|sketch|cartoon)\b', text, re.I):
        return 'cat-image-2'
    if re.search(r'\b(man|woman|person|human|headshot|people)\b', text, re.I) and re.search(r'portrait|photograph|face|close.up', text, re.I):
        return 'cat-image-0'
    return 'cat-image'

def image_title(text):
    clauses = [c.strip() for c in re.split(r'[,\n;]', text) if c.strip()]
    for clause in clauses:
        if len(normalized(clause).split()) >= 4 and not re.fullmatch(r'(highly |ultra |detailed |colored |ink |sharp |focus |cinematic |photorealistic |illustration|painting|render|\d+k| )+', clause, re.I):
            return clause[:100]
    return text[:100]

def image_category(text):
    if re.search(r'\b(product|packaging|bottle|perfume|sneaker|cosmetic)\b|产品|商品|包装', text, re.I):
        return 'cat-image-1'
    if re.search(r'\b(portrait|headshot|photograph\w*)\b|人像|肖像|摄影', text, re.I):
        return 'cat-image-0'
    if re.search(r'\b(illustration|poster|vector|watercolou?r|painting|anime|sketch)\b|插画|海报|绘画', text, re.I):
        return 'cat-image-2'
    return 'cat-image'

def text_category(title, text):
    # Match explicit task titles first. Ambiguous tasks remain at a root category.
    for pattern, category in [
        (r'video|screenwrit|filmmak|storyboard|视频|分镜|编剧', 'cat-video-0'),
        (r'data|statistic|machine learning|数据|统计', 'cat-data'),
        (r'program|develop|code|software|linux|terminal|javascript|python|sql|regex|debug|engineer|\bapi\b|security|html|css|react|web design|powershell|编程|开发|代码|程序|架构', 'cat-software'),
        (r'product|\bux\b|ui design|user experience|产品|交互|用户体验', 'cat-product'),
        (r'market|advertis|seo|social media|brand|sales|business|startup|founder|营销|推广|品牌|创业|商业', 'cat-marketing'),
        (r'teacher|tutor|learn|educat|language|math|study|quiz|course|science|research|academic|学术|学习|教学|教育|科研|论文', 'cat-education'),
        (r'write|writ|poem|poet|story|novel|journal|content|edit|book|写作|小说|诗歌|文案', 'cat-writing'),
        (r'excel|meeting|email|translat|summari|document|presentation|resume|audit|review|办公|翻译|总结|会议|简历', 'cat-office'),
        (r'travel|cook|recipe|fitness|workout|nutrition|career|interview|coach|garden|therap|psycholog|habit|personal|life|旅行|烹饪|健身|生活|心理|面试', 'cat-life'),
    ]:
        if re.search(pattern, title, re.I):
            return category
    if text:
        return text_category(text[:600], '')
    return None

def is_image_task(title, text, kind):
    if kind == 'IMAGE':
        return True
    visual = r'portrait|photograph|cinematic|illustration|watercolor|isometric|poster|3d |image generation|image prompt|海报|人像|生图|插画'
    return bool(re.search(visual, title, re.I) or (STYLE.search(text) and LAYOUT.search(text) and LIGHT.search(text)))

def build(directory, limit=20000, skip_diffusion=False):
    stats = Counter()
    candidates = []
    cols = ['prompt', 'image_name', 'part_id', 'image_nsfw', 'prompt_nsfw', 'width', 'height', 'step', 'cfg', 'seed', 'sampler']
    batches = []
    if not skip_diffusion:
        import pyarrow.parquet as pq
        batches = pq.ParquetFile(directory / 'diffusiondb.parquet').iter_batches(batch_size=16000, columns=cols)
    for batch in batches:
        for row in batch.to_pylist():
            stats['diffusiondb_scanned'] += 1
            score, reason = image_score(row)
            if reason:
                stats['diffusiondb_rejected_' + reason] += 1
            else:
                candidates.append((score, row))
    candidates.sort(key=lambda item: (-item[0], digest(item[1]['prompt'])))
    records, seen, subjects, token_sets = [], set(), set(), set()

    def add(title, content, category, model, reference):
        content = content.strip()
        key = normalized(content)
        if key in seen:
            stats['duplicate'] += 1
            return False
        seen.add(key)
        records.append(dict(id='corpus-' + digest(key)[:24], title=title[:160].strip(), kind='prompt', content=content,
                            excerpt=content[:180], category_id=category, model=model, reference=reference))
        return True

    cc0 = 'https://creativecommons.org/publicdomain/zero/1.0/'
    # Selected by documented heuristics, not a claim of human review or image quality.
    for score, row in candidates:
        words = normalized(row['prompt']).split()
        subject = ' '.join(words[:12])
        bag = ' '.join(sorted(set(w for w in words if not w.isdigit())))
        if subject in subjects or bag in token_sets:
            stats['diffusiondb_near_duplicate'] += 1
            continue
        subjects.add(subject)
        token_sets.add(bag)
        content = row['prompt'].strip()
        title = image_title(content)
        add(title, content, diffusion_category(content), 'Stable Diffusion', dict(
            repository='poloclub/diffusiondb', author='DiffusionDB contributors',
            url='https://huggingface.co/datasets/poloclub/diffusiondb', license='CC0 1.0', license_url=cc0,
            images=[], image_name=row['image_name'], part_id=row['part_id'],
            generation={k: row[k] for k in ('seed', 'step', 'cfg', 'sampler', 'width', 'height')},
            screening='automatic-heuristics-v1', score=score))
        if len(records) >= limit:
            break
    stats['diffusiondb_selected'] = len(records)
    stats['diffusiondb_eligible'] = len(candidates)
    with (directory / 'prompts-chat.csv').open(newline='', encoding='utf-8-sig') as source:
        csv.field_size_limit(8 * 1024 * 1024)
        for row in csv.DictReader(source, strict=True):
            stats['prompts_chat_scanned'] += 1
            title, content = row.get('act') or '', row.get('prompt') or ''
            reason = reject_text(title + '\n' + content)
            if reason or len(normalized(title)) < 4 or row.get('type') not in ('TEXT', 'IMAGE', 'STRUCTURED'):
                stats['prompts_chat_rejected_' + (reason or 'type')] += 1
                continue
            image = is_image_task(title, content, row.get('type'))
            category = image_category(title + ' ' + content) if image else text_category(title, content)
            if not category:
                stats['prompts_chat_rejected_unclear_task'] += 1
                continue
            if add(title, content, category,
                   None if image else 'ChatGPT', dict(repository='f/prompts.chat', author=row.get('contributor') or 'prompts.chat contributors',
                   url='https://github.com/f/prompts.chat/blob/main/prompts.csv', license='CC0 1.0', license_url=cc0, images=[], screening='automatic-heuristics-v1')):
                stats['prompts_chat_selected'] += 1
    # Public README only. Do not request the private CMS used by its generator.
    for block in re.split(r'^### No\. \d+: ', (directory / 'youmind.txt').read_text(), flags=re.M)[1:]:
        stats['youmind_scanned'] += 1
        title = block.split('\n')[0].strip()
        match = re.search(r'#### 📝 提示词\s+```[^\n]*\n([\s\S]*?)\n```', block)
        url = re.search(r'https://youmind.com/zh-CN/nano-banana-pro-prompts\?id=\d+', block)
        author = re.search(r'\*\*作者:\*\* \[([^\]]+)\]', block)
        images = list(dict.fromkeys(re.findall(r'<img src="(https://cms-assets\.youmind\.com/[^"\s]+)"', block)))[:4]
        if not all((match, url, author, images)):
            stats['youmind_rejected_structure'] += 1
            continue
        reason = reject_text(match[1])
        if reason:
            stats['youmind_rejected_' + reason] += 1
            continue
        if add(title, match[1], image_category(title + ' ' + match[1]), 'Nano Banana', dict(
            repository='YouMind-OpenLab/awesome-nano-banana-pro-prompts', author=author[1], url=url[0],
            license='CC BY 4.0', license_url='https://creativecommons.org/licenses/by/4.0/', images=images,
            changes='Original prompt; separate display title and category; no endorsement or image-rights warranty.', screening='automatic-heuristics-v1')):
            stats['youmind_selected'] += 1
    # Image examples first; source order is not claimed to be popularity.
    records.sort(key=lambda r: (not bool(r['reference']['images']), r['model'] == 'Stable Diffusion', r['id']))
    with (directory / 'curated.jsonl').open('w') as dest:
        for record in records:
            dest.write(json.dumps(record, ensure_ascii=False) + '\n')
    report = dict(counts=dict(stats), total=len(records), with_preview=sum(bool(r['reference']['images']) for r in records),
                  categories=dict(Counter(r['category_id'] for r in records)), models=dict(Counter(r['model'] for r in records)),
                  method='Automatic text/metadata screening; not exhaustive human or visual review. No generated filler. External images may fail.',
                  inputs={name: hashlib.file_digest((directory / name).open('rb'), 'sha256').hexdigest()
                          for name in (['prompts-chat.csv', 'youmind.txt'] if skip_diffusion else ['diffusiondb.parquet', 'prompts-chat.csv', 'youmind.txt'])})
    (directory / 'report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps(report, ensure_ascii=False, indent=2))

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    parser.add_argument('--limit', type=int, default=20000)
    parser.add_argument('--skip-diffusion', action='store_true', help='Process GitHub snapshots while the larger dataset is unavailable')
    args = parser.parse_args()
    if not 1 <= args.limit <= 50000:
        parser.error('limit must be between 1 and 50000')
    build(args.directory, args.limit, args.skip_diffusion)
