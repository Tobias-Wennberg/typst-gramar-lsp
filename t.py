import json

# Your JSON data
json_data = '''
{
    "lt_enabled": true, 
    "lt_api_hostname": "http://127.0.0.1", 
    "lt_api_port": "8081" 
}
'''

try:
    # Parse JSON
    data = json.loads(json_data)
    print(data)
except json.JSONDecodeError as e:
    # Handle JSON decoding error
    print("Error decoding JSON:", e)

