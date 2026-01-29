import requests

# GET
try:
    response = requests.get('http://localhost:3030/logs')
    print(f"Logs Status: {response.status_code}")
    print(f"Logs Response: {response.text}")
    if response.status_code == 200:
        print("Logs: " + str(response.json()))
except Exception as e:
    print(f"Error fetching logs: {e}")

try:
    response_2 = requests.get('http://localhost:3030/health')
    print(f"Health Status: {response_2.status_code}")
    print(f"Health Response: {response_2.text}")
except Exception as e:
    print(f"Error fetching health: {e}")

try:
    response_3 = requests.get('http://localhost:3030/servers')
    print(f"Servers Status: {response_3.status_code}")
    print(f"Servers Response: {response_3.text}")
    if response_3.status_code == 200:
        print("Servers: " + str(response_3.json()))
except Exception as e:
    print(f"Error fetching servers: {e}")