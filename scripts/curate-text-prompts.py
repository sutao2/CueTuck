"""Extract prompt-only text from pinned public snapshots; no network or execution."""
import argparse
import gzip
import importlib.util
import json
import re
from collections import Counter
from pathlib import Path

spec = importlib.util.spec_from_file_location('base_curator', Path(__file__).with_name('curate-prompt-corpus.py'))
base = importlib.util.module_from_spec(spec)
spec.loader.exec_module(base)

SOURCES = {
    'aj-geddes/useful-ai-prompts': 'AJ Geddes',
    'pnp/copilot-prompts': 'Microsoft 365 & Power Platform Community contributors',
    'jamesmcroft/everyday-prompts': 'James Croft',
}
DIRECTORIES = {
    'academic': 'cat-education', 'education': 'cat-education', 'learning-development': 'cat-education',
    'analysis': 'cat-data', 'data-science': 'cat-data', 'research': 'cat-education',
    'development': 'cat-software', 'technical': 'cat-software', 'coding': 'cat-software',
    'software-development': 'cat-software', 'engineering': 'cat-product',
    'technical-templates': 'cat-software', 'technical-workflows': 'cat-software', 'security': 'cat-software',
    'writing': 'cat-writing', 'creative': 'cat-writing', 'content': 'cat-writing', 'creation': 'cat-writing',
    'marketing': 'cat-marketing', 'sales': 'cat-marketing', 'customer-focused': 'cat-marketing', 'social': 'cat-marketing',
    'product': 'cat-product', 'product-management': 'cat-product', 'creativity-innovation': 'cat-product',
    'administrative': 'cat-office', 'communication': 'cat-office', 'business': 'cat-office',
    'customer-service': 'cat-office', 'decision-making': 'cat-office', 'evaluation-assessment': 'cat-office',
    'management-leadership': 'cat-office', 'operations': 'cat-office', 'optimization': 'cat-office',
    'planning': 'cat-office', 'problem-solving': 'cat-office', 'project-management': 'cat-office',
    'human-resources': 'cat-office', 'quality-assurance': 'cat-software-3',
    'personal-development': 'cat-life', 'career': 'cat-life', 'journaling': 'cat-life',
    'career-development': 'cat-life', 'personal-growth': 'cat-life', 'relationships-communication': 'cat-life',
    'content-creation': 'cat-writing', 'learning-skills': 'cat-education', 'research-workflows': 'cat-education',
    'personal-productivity': 'cat-office', 'supply-chain': 'cat-office', 'government': 'cat-office',
    'knowledge': 'cat-education', 'ideation': 'cat-product', 'evaluation': 'cat-data',
}

def prompt_section(text):
    """Only level-two Prompt/Instructions, ignoring headings inside fenced text."""
    lines, chosen, fence = [], False, None
    for line in text.splitlines():
        marker = re.match(r'^\s*(`{3,}|~{3,})(.*)$', line)
        if marker:
            if fence is None:
                fence = marker[1]
            elif marker[1][0] == fence[0] and len(marker[1]) >= len(fence) and not marker[2].strip():
                fence = None
            if chosen:
                lines.append(line)
            continue
        if fence is None and line.startswith('## '):
            if chosen:
                break
            name = re.sub(r'[^a-z ]', '', line[3:].lower()).strip()
            chosen = name in ('prompt', 'the prompt', 'sample prompt', 'system prompt', 'instructions')
            continue
        if chosen:
            lines.append(line)
    section = '\n'.join(lines).strip().removesuffix('---').strip()
    blocks = re.findall(r'^(`{3,}|~{3,})[^\n]*\n([\s\S]*?)^\1\s*$', section, re.M)
    if len(blocks) == 1:
        return blocks[0][1].strip()
    if blocks or not section or section.startswith('!['):
        return None  # Ambiguous multi-step fragments or no standalone prompt.
    return '\n'.join(re.sub(r'^> ?', '', line) for line in section.splitlines()).strip()

def classify(repository, path, title, body):
    if base.is_image_task(title, body, 'TEXT') or re.search(r'feature.image|image.generat|social.feature.image|image.prompt|portrait|illustration|virtual.background', title + ' ' + path, re.I):
        return None
    for pattern, category in [
        (r'algorithm|debugging|error handling|repository|ci/cd|code review|api design|microservice|database schema|power apps|power automate|adaptive card|github copilot|html|mermaid|devops', 'cat-software'),
        (r'data (analysis|visualization|cleaning)|statistical|dashboard|dataset', 'cat-data'),
        (r'competitive analysis|market research|brand|\bseo\b|sales copy|advertising|marketing|sales pipeline|campaign|audience engagement', 'cat-marketing'),
        (r'lesson|curriculum|learning|teaching|tutor|study guide', 'cat-education'),
        (r'presentation|meeting|email|spreadsheet|\bexcel\b|word document|chief of staff|executive assistant|inbox|calendar|weekly highlights|timesheet|job description', 'cat-office'),
        (r'product (roadmap|requirements|manager|strategy)|user research|ux |ui design', 'cat-product'),
        (r'video content|video script|storyboard', 'cat-video-0'),
    ]:
        if re.search(pattern, title, re.I):
            return category
    if repository != 'pnp/copilot-prompts':
        directory = path.split('/')[1]
        return DIRECTORIES.get(directory) or base.text_category(title, body)
    return base.text_category(title, '')

def build(directory, output):
    seen = set()
    with gzip.open(Path(__file__).resolve().parents[1] / 'samples/community/corpus.jsonl.gz', 'rt') as existing:
        for line in existing:
            seen.add(base.normalized(json.loads(line)['content']))
    stats, categories, selected_sources = Counter(), Counter(), Counter()
    records, manifests = [], []
    output.mkdir(parents=True, exist_ok=True)
    for repository, author in SOURCES.items():
        file = directory / (repository.replace('/', '--') + '.json')
        data = json.loads(file.read_text())
        if data['repository'] != repository or not re.fullmatch('[a-f0-9]{40}', data['commit']) or not data['license'].startswith('MIT License'):
            raise ValueError('Unexpected snapshot identity or license')
        license_file = repository.split('/')[0] + '-text-license.txt'
        (output / license_file).write_text(data['license'])
        manifests.append(dict(repository=repository, commit=data['commit'], snapshot_sha256=base.digest(file.read_text()), license='MIT', license_file=license_file))
        for source in data['files']:
            stats['scanned'] += 1
            if base.digest(source['text']) != source['sha256']:
                raise ValueError('Snapshot hash mismatch')
            title = re.search(r'^# (.+)$', source['text'], re.M)
            body = prompt_section(source['text'])
            if not title or not body:
                stats['rejected_structure'] += 1
                continue
            title = re.sub(r'^[^\w]+', '', title[1]).strip()
            reason = base.reject_text(title + '\n' + body, minimum=160, maximum=15500)
            if re.search(r'healthcare|health-wellness|biotechnology|financ|blockchain|quantum|renewable-energy|space-economy', source['path'], re.I) or re.search(r'diagnos|prescri|therapist|mental.health|insurance.advisor|investment.advi|legal.advi|medical|dental|stock.trad|crypto', title, re.I):
                reason = 'specialist_risk_scope'
            if reason:
                stats['rejected_' + reason] += 1
                continue
            category = classify(repository, source['path'], title, body)
            if not category:
                stats['rejected_unclear_or_visual_task'] += 1
                continue
            key = base.normalized(body)
            if key in seen:
                stats['rejected_duplicate'] += 1
                continue
            seen.add(key)
            records.append(dict(id='corpus-' + base.digest(key)[:24], title=title[:160], kind='prompt', content=body, excerpt=body[:180], category_id=category, model=None,
                reference=dict(repository=repository, author=author, url=f"https://github.com/{repository}/blob/{data['commit']}/{source['path']}",
                    license='MIT', license_url=f"https://github.com/{repository}/blob/{data['commit']}/LICENSE", license_text=data['license'], images=[],
                    screening='text-heuristics-v1', changes='Extracted standalone prompt section; separate title, category and attribution; no appended attribution in body.',
                    requirements='Microsoft Copilot and the referenced workspace/data access may be required.' if repository == 'pnp/copilot-prompts' or re.search(r'\bCopilot\b|my calendar|my emails|\bTeams\b', body) else 'Supply the inputs requested by the prompt; model compatibility not independently verified.')))
            categories[category] += 1
            selected_sources[repository] += 1
    content = ''.join(json.dumps(row, ensure_ascii=False) + '\n' for row in records)
    (output / 'text-corpus.jsonl.gz').write_bytes(gzip.compress(content.encode(), mtime=0))
    report = dict(scanned=stats['scanned'], selected=len(records), rules=dict(stats), categories=dict(categories), sources=dict(selected_sources),
                  note='Automatic heuristic screening plus documented spot checks, not individual human review or effectiveness/safety certification.')
    (output / 'text-corpus-report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    (output / 'text-corpus-sources.json').write_text(json.dumps(dict(sources=manifests, uncompressed_sha256=base.digest(content)), indent=2) + '\n')
    print(json.dumps(report, ensure_ascii=False, indent=2))

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('directory', type=Path)
    parser.add_argument('--output', type=Path, default=Path('samples/community'))
    args = parser.parse_args()
    build(args.directory, args.output)
