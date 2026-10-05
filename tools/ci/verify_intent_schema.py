#!/usr/bin/env python3
import json, pathlib, sys

schema=json.loads(pathlib.Path(sys.argv[1]).read_text())
assert schema["oneOf"][0]["properties"]["kind"]["const"]=="intent"
assert schema["oneOf"][0]["properties"]["choice_mode"]["enum"]==["ask","autopilot"]
assert schema["oneOf"][0]["properties"]["query"]["maxLength"]==512
assert schema["oneOf"][1]["properties"]["kind"]["const"]=="reject"
assert schema["oneOf"][1]["properties"]["reason"]["const"]=="semantic_authority"
for branch in schema["oneOf"]:
    assert branch["additionalProperties"] is False
print("PULQVA_INTENT_SCHEMA_CONTRACT_OK")
