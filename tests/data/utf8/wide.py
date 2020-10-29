len('piñata')   # 6
len('piñata')   # 7
len('🚀')       # 1 (code point), though JS returns 2 (code units)
''.join(map(chr, (0x63, 0x328))) # 'c̨'; length 2 in both Python and JS
