import json

try:
    with open('translations.json', 'r', encoding='utf-8') as f:
        data = json.load(f)
        print("Keys:", list(data.keys()))
        for k in data.keys():
            if isinstance(data[k], dict):
                print(f"Key '{k}' has {len(data[k])} items.")
            else:
                print(f"Key '{k}' is {type(data[k])}")
except Exception as e:
    print(e)
