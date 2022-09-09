// Reads the number of sections & timeDateStamp from the COFF_Header of CalculatorApp.dll
let calc_proc = OS.process("CalculatorApp.exe");
let mod_base = calc_proc.mod("CalculatorApp.dll").base;

// Our native!
native COFFHeader {
    ^ 6,
    sections: UInt16,
    timestamp: UInt32
};

let coff_header = calc_proc.read(COFFHeader, mod_base + 0x40);
print(coff_header);