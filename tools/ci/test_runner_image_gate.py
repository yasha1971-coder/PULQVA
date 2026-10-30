import unittest
from runner_image_gate import require_match,image_identity
def sample(version="20261025.353.1",image="ubuntu-24.04"):
 return [{"group":"Operating System","detail":"Ubuntu\\n24.04.5\\nLTS"},{"group":"Runner Image","detail":"Image: "+image+"\n Version: "+version+"\n Included Software: https://github.com/actions/runner-images/blob/ubuntu24/20261025.353/images/ubuntu/Ubuntu2404-Readme.md\n Image Release: https://github.com/actions/runner-images/releases/tag/ubuntu24%2F20261025.353"}]
class GateTests(unittest.TestCase):
 def test_exact_official_metadata_passes(self):self.assertEqual(require_match(sample(),"ubuntu-24.04","20261025.353.1")["Version"],"20261025.353.1")
 def test_version_mismatch_fails(self):
  with self.assertRaises(ValueError):require_match(sample("20261026.354.1"),"ubuntu-24.04","20261025.353.1")
 def test_other_os_fails(self):
  with self.assertRaises(ValueError):require_match(sample(image="ubuntu-22.04"),"ubuntu-24.04","20261025.353.1")
 def test_missing_group_fails(self):
  with self.assertRaises(ValueError):image_identity([])
 def test_duplicate_group_fails(self):
  with self.assertRaises(ValueError):image_identity(sample()+sample())
 def test_duplicate_version_fails(self):
  d=sample();d[1]["detail"]+="\nVersion: 20261025.353.1"
  with self.assertRaises(ValueError):image_identity(d)
 def test_unknown_shape_fails(self):
  with self.assertRaises(ValueError):image_identity({})
 def test_missing_version_fails(self):
  with self.assertRaises(ValueError):image_identity([{"group":"Runner Image","detail":"Image: ubuntu-24.04"}])
if __name__=="__main__":unittest.main()
