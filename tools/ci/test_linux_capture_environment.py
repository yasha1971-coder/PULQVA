import unittest
from linux_capture_environment import capture_environment
class CaptureEnvironmentTests(unittest.TestCase):
    def test_xvfb_cookie_path_survives_isolated_home(self):
        env=capture_environment({"isolated_environment":{"HOME":"/owned/home"}},
            {"HOME":"/not-owned/home","DISPLAY":":99","XAUTHORITY":"/owned/xvfb/Xauthority"})
        self.assertEqual(env["XAUTHORITY"],"/owned/xvfb/Xauthority")
        self.assertEqual(env["DISPLAY"],":99")
        self.assertEqual(env["HOME"],"/owned/home")
    def test_non_allowlisted_values_are_not_inherited(self):
        env=capture_environment({},{"PRIVATE_SENTINEL":"do-not-capture","PATH":"/usr/bin"})
        self.assertNotIn("PRIVATE_SENTINEL",env)
        self.assertEqual(env["PATH"],"/usr/bin")
    def test_authorization_is_not_fabricated_when_absent(self):
        env=capture_environment({},{"DISPLAY":":99"})
        self.assertNotIn("XAUTHORITY",env)
        self.assertEqual(env["PULQVA_TOR_DIAGNOSTICS"],"1")
