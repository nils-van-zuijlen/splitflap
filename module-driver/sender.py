import serial
from serial.tools import list_ports


def transform(value: str) -> bytes:
    value = value.upper()

    out = b"\xF0"

    for char in value:
        try:
            index = LETTERS.index(char)
            out += index.to_bytes()
        except ValueError:
            # not a valid letter
            if char == "\\":  # reset char
                out += b"\xFE"
            elif char == "@":  # reset all
                out += b"\xFF"

    return out


class Sender:
    def __init__(self):
        ports = list_ports.comports()

        for port in ports:
            if port.description == "Split flap":
                break
        else:
            raise RuntimeError("No split flap display serial found.")

        self.output = serial.Serial(port.device)
        print("Serial port found and initialized.")

    def send(self, word: str):
        self.output.write(transform(word))


LETTERS = " ABCDEFGHIJKLMNOPQRSTUVWXYZ$&#0123456789:.-?!"


if __name__ == "__main__":
    sender = Sender()
    while entered := input(">>> "):
        sender.send(entered)
