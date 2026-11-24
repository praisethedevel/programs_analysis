#!/usr/bin/env python3
import sys

def parse_security_txt(so_path):
    with open(so_path, 'rb') as f:
        data = f.read()
    
    start_tag = b"=======BEGIN SECURITY.TXT V1=======\x00"
    end_tag = b"=======END SECURITY.TXT V1=======\x00"
    
    start = data.find(start_tag)
    if start == -1:
        return None, "Start tag not found"
    
    start += len(start_tag)
    end = data.find(end_tag, start)
    if end == -1:
        return None, "End tag not found"
        
    raw_payload = data[start:end]
    entries = raw_payload.split(b"\x00")
    
    # Filter trailing empties
    while entries and not entries[-1]:
        entries.pop()
        
    if len(entries) % 2 != 0:
        return None, f"Odd number of entries ({len(entries)}), malformed key-value pairs"
        
    result = {}
    for i in range(0, len(entries), 2):
        k = entries[i].decode('utf-8', errors='replace')
        v = entries[i+1].decode('utf-8', errors='replace')
        result[k] = v
        
    return result, None

def main():
    programs = [
        ("CustomToken", "/home/joe/programs_analysis/CustomToken.so"),
        ("CustomSysclock", "/home/joe/programs_analysis/CustomSysclock.so"),
        ("CustomRAM", "/home/joe/programs_analysis/CustomRAM.so"),
    ]
    
    print("=" * 65)
    print("SOLANA SBPF SECURITY.TXT VERIFICATION ENGINE")
    print("=" * 65)
    
    mandatory_fields = ["name", "contacts", "policy", "source_code"]
    all_passed = True
    
    for name, path in programs:
        print(f"\n[*] Auditing {name} ({path})...")
        parsed, err = parse_security_txt(path)
        if err:
            print(f"  ❌ FAILED: {err}")
            all_passed = False
            continue
            
        print("  ✅ Found valid Neodyme SECURITY.TXT v1 payload:")
        for k, v in parsed.items():
            print(f"     - {k:<20}: {v}")
            
        missing = [f for f in mandatory_fields if f not in parsed]
        if missing:
            print(f"  ❌ Missing mandatory fields: {missing}")
            all_passed = False
        else:
            print("  ✅ All mandatory fields present and compliant!")
            
    print("\n" + "=" * 65)
    if all_passed:
        print("🎉 ALL 3 SBPF BINARIES PASSED SECURITY.TXT VERIFICATION 100%!")
    else:
        print("❌ SOME BINARIES FAILED VERIFICATION")
        sys.exit(1)
    print("=" * 65)

if __name__ == "__main__":
    main()
