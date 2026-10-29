import json, pathlib, tempfile, unittest
from diagnostic_input_guard import PREFIX, snapshot, verify
class GuardTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.root=pathlib.Path(self.temp.name)
        (self.root/"source.rs").write_text("frozen")
        self.names=["source.rs"];self.before=snapshot(self.root,self.names)
    def tearDown(self): self.temp.cleanup()
    def emit(self,name,data="{}"):
        path=self.root/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_text(data);return path
    def test_generated_schema_is_hashed(self):
        n=PREFIX+"linux-schema.json";self.emit(n)
        result=verify(self.root,self.before,self.names,[n],"linux")
        self.assertTrue(result["tracked_inputs_unchanged"]);self.assertEqual(result["generated"][n]["bytes"],2)
    def test_tracked_source_edit_rejected(self):
        (self.root/"source.rs").write_text("changed")
        with self.assertRaisesRegex(ValueError,"tracked source"):verify(self.root,self.before,self.names,[],"linux")
    def test_unknown_output_rejected(self):
        self.emit("unexpected.rs")
        with self.assertRaisesRegex(ValueError,"unexpected untracked"):verify(self.root,self.before,self.names,["unexpected.rs"],"linux")
    def test_wrong_platform_schema_rejected(self):
        n=PREFIX+"windows-schema.json";self.emit(n)
        with self.assertRaises(ValueError):verify(self.root,self.before,self.names,[n],"linux")
    def test_invalid_json_rejected(self):
        n=PREFIX+"capabilities.json";self.emit(n,"not-json")
        with self.assertRaises(ValueError):verify(self.root,self.before,self.names,[n],"linux")
    def test_removed_tracked_input_rejected(self):
        (self.root/"source.rs").unlink()
        with self.assertRaises(ValueError):verify(self.root,self.before,self.names,[],"linux")
    def test_symlink_detection(self):
        target=self.emit("target.json");link=self.root/(PREFIX+"capabilities.json");link.parent.mkdir(parents=True,exist_ok=True)
        try:link.symlink_to(target)
        except OSError:self.skipTest("host does not permit symlinks; CI Linux covers this probe")
        with self.assertRaisesRegex(ValueError,"symlink"):verify(self.root,self.before,self.names,[PREFIX+"capabilities.json"],"linux")
if __name__=="__main__":unittest.main(verbosity=2)
