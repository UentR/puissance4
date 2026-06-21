def mask(code):
    for i in range(5, -1, -1):
        for j in range(6, -1, -1):
            print(code&(1<<(i*7+j))!=0, end=" ")
        print()
        


code16 = 0x0000001020408102

mask(code16)
