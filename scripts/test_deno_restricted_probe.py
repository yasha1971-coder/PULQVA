import subprocess
import unittest
from deno_restricted_probe import FLAGS, validate_result


class ProbeValidationTests(unittest.TestCase):
    def result(self, code=0, out='{"denied":"net"}', err=""):
        return subprocess.CompletedProcess([], code, out, err)

    def test_exact_denial(self):
        validate_result(self.result(), {"denied": "net"})

    def test_generic_failure_not_accepted(self):
        for result in [self.result(code=1), self.result(err="connection failed"),
                       self.result(out='{"denied":"read"}'), self.result(out="not json")]:
            with self.subTest(result=result), self.assertRaises((ValueError, TypeError)):
                validate_result(result, {"denied": "net"})

    def test_no_permission_grants(self):
        self.assertFalse(any(flag.startswith("--allow") for flag in FLAGS))
        for required in ["--no-remote", "--no-npm", "--no-config", "--cached-only", "--no-prompt"]:
            self.assertIn(required, FLAGS)


if __name__ == "__main__":
    unittest.main()
