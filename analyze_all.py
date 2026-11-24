import subprocess
import glob
import os
import json

workdir = "/home/joe/programs_analysis"
soldecomp = "/home/joe/soldecomp/soldecomp.py"
solazy = "/home/joe/.cargo/bin/sol-azy"

files = sorted(glob.glob(os.path.join(workdir, "*.so")))

for f in files:
    base = os.path.basename(f)
    prog_key = base.replace(".so", "")
    print("=" * 60)
    print(f"ANALYZING: {base} ({os.path.getsize(f)} bytes)")
    print("=" * 60)
    
    # 1. soldecomp detect-framework
    res_detect = subprocess.run(["python3", soldecomp, "detect-framework", f], capture_output=True, text=True)
    detect_json = {}
    try:
        detect_json = json.loads(res_detect.stdout)
    except Exception as e:
        detect_json = {"raw": res_detect.stdout, "err": str(e)}
    
    print("--- [soldecomp detect-framework] ---")
    print("Framework:", detect_json.get("framework"))
    print("sBPF:", detect_json.get("sbpf"))
    print("Signals:", detect_json.get("signals"))
    insts = detect_json.get("instructions_from_binary", [])
    print(f"Instructions detected: {len(insts)}")
    for inst in insts[:10]:
        print("  -", inst)

    # 2. sol-azy reverse (disassembly)
    out_dir = os.path.join(workdir, f"solazy_{prog_key}")
    os.makedirs(out_dir, exist_ok=True)
    res_solazy = subprocess.run([solazy, "reverse", "--mode", "disass", "--out-dir", out_dir, "--bytecodes-file", f], capture_output=True, text=True)
    disasm_file = os.path.join(out_dir, "disassembly.out")
    
    lines_count = 0
    syscalls = {}
    if os.path.exists(disasm_file):
        with open(disasm_file, "r", errors="ignore") as df:
            for line in df:
                lines_count += 1
                if "syscall " in line:
                    sc = line.split("syscall ")[1].strip().split()[0]
                    syscalls[sc] = syscalls.get(sc, 0) + 1
    
    print("\n--- [sol-azy reverse] ---")
    print(f"Disassembly lines: {lines_count}")
    print("Syscalls used:", sorted(syscalls.items(), key=lambda x: x[1], reverse=True))

    # 3. String probes
    strings_res = subprocess.run(["strings", f], capture_output=True, text=True)
    strs = strings_res.stdout.splitlines()
    has_anchor = any("anchor" in s.lower() for s in strs)
    has_pinocchio = any("pinocchio" in s.lower() for s in strs)
    log_messages = [s for s in strs if "Instruction:" in s or "Program log:" in s or "panicked at" in s][:10]
    print("\n--- [string probes] ---")
    print("Has Anchor artifacts:", has_anchor)
    print("Has Pinocchio artifacts:", has_pinocchio)
    print("Sample log/panic strings:", log_messages)
    print("\n")
