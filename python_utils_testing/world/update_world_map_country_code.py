import json
import os

def load_country_codes():
    """Load country codes mapping from codes.json"""
    with open('codes.json', 'r', encoding='utf-8') as f:
        codes_data = json.load(f)
    
    # Create a mapping from country name to country code
    name_to_code = {}
    for country in codes_data:
        name_to_code[country['name']] = country['code']
    
    return name_to_code

def update_world_map():
    """Update world.json to use country codes instead of names"""
    
    # Load country codes mapping
    name_to_code = load_country_codes()
    
    # Load world map data
    with open('world.json', 'r', encoding='utf-8') as f:
        world_data = json.load(f)
    
    unmatched_countries = set()
    matched_count = 0
    
    # Update each feature's properties
    for feature in world_data['features']:
        if 'properties' in feature and 'name' in feature['properties']:
            country_name = feature['properties']['name']
            
            # Try to find exact match first
            if country_name in name_to_code:
                feature['properties']['name'] = name_to_code[country_name]
                matched_count += 1
            else:
                # Try case-insensitive match
                found = False
                for name, code in name_to_code.items():
                    if name.lower() == country_name.lower():
                        feature['properties']['name'] = code
                        matched_count += 1
                        found = True
                        break
                
                if not found:
                    # Try partial matching for common variations
                    found = False
                    country_lower = country_name.lower()
                    
                    # Handle common variations
                    variations = {
                        'united states': 'United States',
                        'usa': 'United States',
                        'uk': 'United Kingdom',
                        'russia': 'Russian Federation',
                        'south korea': 'Korea, Republic of',
                        'north korea': "Korea, Democratic People'S Republic of",
                        'iran': 'Iran, Islamic Republic Of',
                        'syria': 'Syrian Arab Republic',
                        'venezuela': 'Venezuela',
                        'macedonia': 'Macedonia, The Former Yugoslav Republic of',
                        'congo': 'Congo',
                        'democratic republic of congo': 'Congo, The Democratic Republic of the',
                        'ivory coast': "Cote D'Ivoire",
                        'vatican': 'Holy See (Vatican City State)',
                        'taiwan': 'Taiwan, Province of China',
                        'palestine': 'Palestinian Territory, Occupied'
                    }
                    
                    if country_lower in variations:
                        actual_name = variations[country_lower]
                        if actual_name in name_to_code:
                            feature['properties']['name'] = name_to_code[actual_name]
                            matched_count += 1
                            found = True
                    
                    if not found:
                        unmatched_countries.add(country_name)
    
    # Save updated world map
    with open('world_updated.json', 'w', encoding='utf-8') as f:
        json.dump(world_data, f, indent=2, ensure_ascii=False)
    
    # Print results
    print(f"✅ Successfully updated world map!")
    print(f"📊 Matched {matched_count} countries")
    print(f"📁 Updated file saved as: world_updated.json")
    
    if unmatched_countries:
        print(f"\n❌ Unmatched countries ({len(unmatched_countries)}):")
        for country in sorted(unmatched_countries):
            print(f"   - {country}")
        
        print(f"\n💡 Please manually check these countries in codes.json:")
        print(f"   They might have different names or be missing from the codes list.")
    else:
        print(f"\n🎉 All countries were successfully matched!")

if __name__ == "__main__":
    # Change to the script's directory
    script_dir = os.path.dirname(os.path.abspath(__file__))
    os.chdir(script_dir)
    
    try:
        update_world_map()
    except FileNotFoundError as e:
        print(f"❌ Error: {e}")
        print("Make sure both world.json and codes.json are in the same directory as this script.")
    except Exception as e:
        print(f"❌ Unexpected error: {e}")
