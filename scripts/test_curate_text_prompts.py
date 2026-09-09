import importlib.util
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location('text_curator', Path(__file__).with_name('curate-text-prompts.py'))
curator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(curator)

class ExtractionTests(unittest.TestCase):
    def test_extracts_prompt_without_example_metadata_or_attribution(self):
        text = '# Title\n## Metadata\nCopyright notice\n## Prompt\n```text\nDo this task.\n## A heading inside the prompt\nOutput a table.\n```\n---\n## Example Usage\nFake answer\n'
        self.assertEqual(curator.prompt_section(text), 'Do this task.\n## A heading inside the prompt\nOutput a table.')

    def test_missing_and_ambiguous_sections_are_not_imported(self):
        self.assertIsNone(curator.prompt_section('# Links\n## Examples\nNot a prompt'))
        self.assertIsNone(curator.prompt_section('## Prompt\n```\nFirst\n```\n```\nSecond\n```'))

    def test_quote_format_and_directory_classification(self):
        self.assertEqual(curator.prompt_section('## Prompt\n> Summarize my meeting.\n> Return action items.\n## More\nFooter'), 'Summarize my meeting.\nReturn action items.')
        self.assertEqual(curator.classify('aj-geddes/useful-ai-prompts', 'prompts/education/lesson.md', 'Lesson Plan', 'Text'), 'cat-education')
        self.assertEqual(curator.classify('jamesmcroft/everyday-prompts', 'prompts/content/post.md', 'Write Article', 'Text'), 'cat-writing')
        self.assertIsNone(curator.classify('jamesmcroft/everyday-prompts', 'prompts/content/feature-image.md', 'Feature Image', 'Text'))

    def test_explicit_domains_are_not_overridden_by_incidental_body_words(self):
        repo = 'aj-geddes/useful-ai-prompts'
        self.assertEqual(curator.classify(repo, 'prompts/career-development/self-discipline.md', 'Self Discipline Developer', 'Help developers build habits'), 'cat-life')
        self.assertEqual(curator.classify(repo, 'prompts/personal-productivity/remote.md', 'Remote Work Optimizer', 'Schedule video calls'), 'cat-office')
        self.assertEqual(curator.classify(repo, 'prompts/analysis/market.md', 'Competitive Analysis Expert', 'Analyze datasets'), 'cat-marketing')
        self.assertEqual(curator.classify(repo, 'prompts/problem-solving/debug.md', 'Debugging Expert', 'Solve problems'), 'cat-software')
        self.assertEqual(curator.classify(repo, 'prompts/academic/research.md', 'Research Excellence Scientist', 'Research planning'), 'cat-education')
        self.assertIsNone(curator.classify('pnp/copilot-prompts', 'samples/background/README.md', 'Virtual Background', 'Create a background'))

if __name__ == '__main__':
    unittest.main()
