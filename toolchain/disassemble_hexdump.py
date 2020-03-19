#!/usr/bin/env python3

import struct
import os
import tempfile

list = [0x55, 0x48, 0x89, 0xe5, 0xb8, 0xd2, 0x4, 0x0, 0x0, 0x5d, 0xc3, 0x55, 0x48, 0x89, 0xe5, 0x31, 0xc0, 0x8b, 0xf, 0x39, 0xc8, 0x72, 0x2, 0xf, 0xb, 0x89, 0xc0, 0x48, 0x8b, 0xf, 0xba, 0xd2, 0x4, 0x0, 0x0, 0x48, 0xf, 0xaf, 0xc2, 0x48, 0x8b, 0x44, 0x1, 0x78, 0xff, 0xd0, 0x5d, 0xc3]

out = tempfile.NamedTemporaryFile(mode = 'wb')
out.write(struct.pack(f'{len(list)}B', *list))
out.flush()
os.system(f'objdump -b binary -D -m i386:x86-64 {out.name}')
out.close()
