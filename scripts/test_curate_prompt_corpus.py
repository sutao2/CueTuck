import importlib.util
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location('curator', Path(__file__).with_name('curate-prompt-corpus.py'))
curator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(curator)

class CuratorTests(unittest.TestCase):
    def row(self):
        return dict(prompt='A quiet mountain valley with a winding river, soft golden hour lighting, wide angle perspective, detailed watercolor illustration with natural colors and a balanced foreground composition.', image_nsfw=0.01, prompt_nsfw=0.01, width=512, height=768, step=50, cfg=7)

    def test_safety_scores_fail_closed(self):
        self.assertIsNotNone(curator.image_score(self.row())[0])
        for bad in [None, float('nan'), -0.1, 0.5]:
            row = self.row(); row['image_nsfw'] = bad
            self.assertEqual(curator.image_score(row)[1], 'safety_score')

    def test_junk_and_risk_are_not_selected(self):
        for text in ['', 'test', 'https://example.com ' * 10, 'nsfw portrait ' * 10]:
            self.assertIsNotNone(curator.reject_text(text))

    def test_categories_do_not_mislabel_scenery_as_a_portrait(self):
        self.assertEqual(curator.image_category('mountain landscape wide angle'), 'cat-image')
        self.assertEqual(curator.image_category('product perfume bottle portrait'), 'cat-image-1')
        self.assertEqual(curator.image_category('watercolor illustration'), 'cat-image-2')
        self.assertEqual(curator.diffusion_category('landscape photography in cinematic light'), 'cat-image')
        self.assertEqual(curator.diffusion_category('portrait painting of a woman'), 'cat-image-2')
        self.assertEqual(curator.image_title('sharp, highly detailed, a mountain valley in sunlight'), 'a mountain valley in sunlight')

    def test_repeated_fragments_and_nonsense_parameters_are_rejected(self):
        row = self.row(); row['prompt'] += ' ' + row['prompt']
        self.assertEqual(curator.image_score(row)[1], 'keyword_stuffing')
        row = self.row(); row['prompt'] += ', 178800mm lens'
        self.assertEqual(curator.image_score(row)[1], 'malformed_parameters')

    def test_normalization_and_task_categories(self):
        self.assertEqual(curator.normalized('Hello,  WORLD!'), curator.normalized('hello world'))
        self.assertEqual(curator.text_category('Python developer', ''), 'cat-software')
        self.assertEqual(curator.text_category('Data analyst', ''), 'cat-data')

if __name__ == '__main__':
    unittest.main()
