import serial
from serial.tools import list_ports

ports = list_ports.comports()

for port in ports:
    if port.description == 'Split flap':
        break
else:
    print("No split flap display serial found.")
    exit(1)

output = serial.Serial(port.device)
print("Serial port found and initialized.")

LETTERS = " ABCDEFGHIJKLMNOPQRSTUVWXYZ$&#0123456789:.-?!"


def transform(value: str) -> bytes:
    value = value.upper()

    out = b'\xF0'

    for char in value:
        try:
            index = LETTERS.index(char)
            out += index.to_bytes()
        except ValueError:
            # not a valid letter
            if char == '\\':  # reset char
                out += b'\xFE'
            elif char == '@':  # reset all
                out += b'\xFF'

    return out


while entered := input(">>> "):
    output.write(transform(entered))
