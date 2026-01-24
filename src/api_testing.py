import requests

# GET
response = requests.get('http://localhost:3030/logs')
print("Logs: " + str(response.json()))

response_2 = requests.get('http://localhost:3030/health')
print("Health: " + response_2.text)