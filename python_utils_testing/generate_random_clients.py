import random
import uuid
import json

# Sample country codes
country_codes = [
    "US", "GB", "DE", "FR", "JP", "KR", "BR", "IN", "RU", "CA",
    "IT", "AU", "NL", "SE", "NO", "CH", "ES", "MX", "ZA", "CN"
]

# Some sample data pools
cpu_models = [
    "Intel Core i9-13900K", "AMD Ryzen 9 7950X", "Intel Core i7-13800H",
    "Apple M2 Max", "Intel Xeon W-2295", "AMD Ryzen 7 5800X"
]

gpus = [
    "NVIDIA RTX 4090", "AMD Radeon RX 7900 XTX", "Intel Iris Xe",
    "Microsoft Basic Render Driver", "NVIDIA RTX 2000 Ada Generation Laptop GPU"
]

avs = [
    "Windows Defender", "McAfee", "Norton", "Kaspersky", "Bitdefender", "Avast"
]

oses = [
    "Windows 10 (19045)", "Windows 11 (22621)", "Ubuntu 22.04", "macOS Ventura", "Fedora 38"
]

storages = [
    "512 GB SSD", "1 TB HDD", "256 GB NVMe", "2 TB SSD", "1.5 TB SATA"
]

groups = ["Default", "Admins", "Guests", "Beta Testers"]

def random_ram():
    return f"{random.uniform(8, 128):.1f} GB"

def generate_client():
    country_code = random.choice(country_codes)
    return {
        "addr": f"127.0.0.1:{random.randint(10000, 60000)}",
        "country_code": country_code,
        "cpu": random.choice(cpu_models),
        "disconnected": random.choice([True, False]),  # <-- fixed
        "displays": random.randint(1, 3),
        "gpus": random.sample(gpus, random.randint(1, 3)),
        "group": random.choice(groups),
        "hostname": f"host-{uuid.uuid4().hex[:6]}",
        "installed_avs": random.sample(avs, random.randint(1, 2)),
        "is_elevated": random.choice([True, False]),
        "os": random.choice(oses),
        "ram": random_ram(),
        "reverse_proxy_port": "" if random.random() < 0.5 else str(random.randint(1000, 9999)),
        "storage": random.sample(storages, random.randint(1, 3)),
        "username": f"user{random.randint(1, 100)}",
        "uuidv4": str(uuid.uuid4())
    }

def generate_clients(n=50):
    return [generate_client() for _ in range(n)]

if __name__ == "__main__":
    clients = generate_clients(50)

    # Write to file
    with open("test_clients.json", "w", encoding="utf-8") as f:
        json.dump(clients, f, indent=4)

    print(f"✅ Successfully wrote {len(clients)} clients to test_clients.json")
