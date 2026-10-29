#!/usr/bin/env python3
"""Validate the bridge interface and Rust subprocess; no D-Bus or NFC device needed."""
import hashlib
import importlib.util
import subprocess
import tempfile
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('bridge',ROOT/'adapters/nfcd/bridge.py')
module=importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

def main():
    # Generate the existing deterministic, expired public fixture; no local assets or keys.
    payload=subprocess.run(
        ['cargo','run','--quiet','--locked','-p','tap-cli','--example','fixture'],
        cwd=ROOT,stdin=subprocess.DEVNULL,capture_output=True,check=True,timeout=120,
    ).stdout
    with tempfile.TemporaryDirectory() as directory:
        output=Path(directory)/'unverified.json'
        app=module.HostApp(str(ROOT/'target/debug/linux-tap'),str(output))
        try:
            interface=app.introspect()
            process=next(m for m in interface.methods if m.name=='Process')
            assert process.in_signature=='oyyyyayu'
            assert process.out_signature=='ayyyu'
            app.select()
            opening=len(payload).to_bytes(2,'big')+hashlib.blake2b(payload,digest_size=32).digest()
            assert app.exchange((bytes([0x80,0x10,0,0,34])+opening).hex()).endswith(b'\x90\x00')
            for offset in range(0,len(payload),220):
                chunk=payload[offset:offset+220]
                apdu=bytes([0x80,0x11])+offset.to_bytes(2,'big')+bytes([len(chunk)])+chunk
                assert app.exchange(apdu.hex()).endswith(b'\x90\x00')
            assert not output.exists()
            response=app.exchange('8012000000')
            assert response==hashlib.blake2b(payload,digest_size=32).digest()+b'\x90\x00'
            assert output.read_bytes()==payload
            assert app.exchange('8012000000')==response
            print('PASS nfcd interface signatures and Rust pipe bridge; physical hardware and D-Bus registration not tested')
        finally:
            app.close()

if __name__=='__main__':
    main()
