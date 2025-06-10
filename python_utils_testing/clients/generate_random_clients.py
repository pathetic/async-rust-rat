import random
import uuid
import json

# Sample country codes
country_codes = [
    "US", "GB", "DE", "FR", "JP", "KR", "BR", "IN", "RU", "CA",
    "IT", "AU", "NL", "SE", "NO", "CH", "ES", "MX", "ZA", "CN", "PRIVATE"
]

# Sample data pools for new format
bios_manufacturers = ["Microsoft Corporation", "Dell Inc.", "HP Inc.", "Lenovo", "ASUS", "Acer Inc."]
bios_versions = ["21.109.143", "2.15.0", "1.12.2", "3.02", "F.49", "308"]

cpu_names = [
    "13th Gen Intel(R) Core(TM) i7-13800H",
    "12th Gen Intel(R) Core(TM) i9-12900K", 
    "AMD Ryzen 9 7950X",
    "11th Gen Intel(R) Core(TM) i5-1135G7",
    "AMD Ryzen 7 5800X",
    "Intel(R) Core(TM) i7-10700K"
]

cpu_descriptions = [
    "Intel64 Family 6 Model 186 Stepping 2",
    "Intel64 Family 6 Model 167 Stepping 1", 
    "AMD64 Family 25 Model 97 Stepping 2",
    "Intel64 Family 6 Model 140 Stepping 1"
]

cpu_manufacturers = ["GenuineIntel", "AuthenticAMD"]

gpu_names = [
    "Intel(R) Iris(R) Xe Graphics",
    "NVIDIA RTX 2000 Ada Generation Laptop GPU",
    "NVIDIA GeForce RTX 4090",
    "AMD Radeon RX 7900 XTX",
    "NVIDIA GeForce GTX 1660 Ti",
    "AMD Radeon RX 6700 XT"
]

drive_models = [
    "SDCPNRZ-2T00-1124-WD",
    "Samsung SSD 980 PRO 1TB",
    "WDC WD10EZEX-08WN4A0",
    "ST2000DM008-2FR102",
    "Crucial MX500 500GB"
]

antivirus_options = [
    ["Windows Defender"],
    ["Windows Defender", "McAfee"],
    ["Norton"],
    ["Kaspersky"],
    ["Bitdefender"],
    ["Avast", "Malwarebytes"]
]

os_names = [
    "Windows 10 Pro", "Windows 11 Pro", "Windows 10 Home", 
    "Windows 11 Home", "Windows Server 2022"
]

os_versions = ["10 (19045)", "11 (22621)", "10 (19044)", "11 (22000)"]

system_manufacturers = [
    "Microsoft Corporation", "Dell Inc.", "HP Inc.", "Lenovo", 
    "ASUS", "Acer Inc.", "MSI"
]

system_models = [
    "Surface Laptop Studio 2", "OptiPlex 7090", "EliteBook 850 G8",
    "ThinkPad X1 Carbon", "ROG Strix G15", "Aspire 5"
]

groups = ["Default", "Admins", "Guests", "Beta Testers", "Power Users"]

def generate_serial_number():
    """Generate a random serial number"""
    return ''.join(random.choices('0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ', k=random.randint(10, 15)))

def generate_mac_address():
    """Generate a random MAC address"""
    return ''.join(random.choices('0123456789ABCDEF', k=12))

def generate_volume_serial():
    """Generate a random volume serial"""
    return ''.join(random.choices('0123456789ABCDEF', k=7))

def generate_bios():
    version = random.choice(bios_versions)
    return {
        "description": version,
        "manufacturer": random.choice(bios_manufacturers),
        "serial_number": generate_serial_number(),
        "version": version
    }

def generate_cpu():
    return {
        "clock_speed_mhz": random.randint(1800, 5000),
        "cpu_name": random.choice(cpu_names),
        "description": random.choice(cpu_descriptions),
        "logical_processors": random.choice([4, 6, 8, 12, 16, 20, 24]),
        "manufacturer": random.choice(cpu_manufacturers),
        "processor_family": str(random.randint(150, 250))
    }

def generate_data():
    return {
        "addr": f"127.0.0.1:{random.randint(10000, 60000)}",
        "country_code": random.choice(country_codes),
        "disconnected": None if random.random() < 0.7 else random.choice([True, False]),
        "group": random.choice(groups),
        "reverse_proxy_port": "" if random.random() < 0.6 else str(random.randint(1000, 9999)),
        "uuidv4": str(uuid.uuid4())
    }

def generate_drives():
    num_drives = random.randint(1, 3)
    drives = []
    for _ in range(num_drives):
        drives.append({
            "model": random.choice(drive_models),
            "size_gb": round(random.uniform(250, 4000), 2)
        })
    return drives

def generate_gpus():
    num_gpus = random.randint(1, 2)
    gpus = []
    selected_gpus = random.sample(gpu_names, min(num_gpus, len(gpu_names)))
    
    for gpu_name in selected_gpus:
        gpus.append({
            "driver_version": f"{random.randint(30, 35)}.{random.randint(0, 9)}.{random.randint(100, 999)}.{random.randint(1000, 9999)}",
            "name": gpu_name
        })
    return gpus

def generate_ram():
    total = round(random.uniform(8, 128), 2)
    used = round(random.uniform(2, total * 0.8), 2)
    return {
        "total_gb": total,
        "used_gb": used
    }

def generate_security():
    return {
        "antivirus_names": random.choice(antivirus_options),
        "firewall_enabled": random.choice([True, False])
    }

def generate_system():
    return {
        "is_elevated": random.choice([True, False]),
        "machine_name": f"host-{random.choice(['pc', 'laptop', 'desktop', 'workstation'])}-{random.randint(1, 999):03d}",
        "os_full_name": random.choice(os_names),
        "os_serial_number": f"{random.randint(10000, 99999)}-{random.randint(10000, 99999)}-{random.randint(10000, 99999)}-AAOEM",
        "os_version": random.choice(os_versions),
        "system_manufacturer": random.choice(system_manufacturers),
        "system_model": random.choice(system_models),
        "username": f"user{random.randint(1, 100)}"
    }

def generate_unique():
    return {
        "mac_address": generate_mac_address(),
        "volume_serial": generate_volume_serial()
    }

def generate_client():
    return {
        "bios": generate_bios(),
        "cpu": generate_cpu(),
        "data": generate_data(),
        "displays": random.randint(1, 4),
        "drives": generate_drives(),
        "gpus": generate_gpus(),
        "ram": generate_ram(),
        "security": generate_security(),
        "system": generate_system(),
        "unique": generate_unique()
    }

def generate_clients(n=50):
    return [generate_client() for _ in range(n)]

if __name__ == "__main__":
    clients = generate_clients(50)

    # Write to file
    with open("test_clients.json", "w", encoding="utf-8") as f:
        json.dump(clients, f, indent=2)

    print(f"✅ Successfully wrote {len(clients)} clients to test_clients.json")
