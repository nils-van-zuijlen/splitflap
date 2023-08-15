import argparse
import sender
import time

parser = argparse.ArgumentParser(
    prog='Split flap tester',
)
parser.add_argument('module_count', metavar='N_MODULES', type=int, help='Number of modules to test')
parser.add_argument('-d', '--delay', type=int, help='Seconds to wait between letters', default=5)

args = parser.parse_args()
print(f"Testing {args.module_count} modules.")

s = sender.Sender()

print('Resetting')
s.send('@')
time.sleep(25)

for letter in sender.LETTERS:
    print(f'Sending letter \'{letter}\'')
    s.send(letter * args.module_count)
    time.sleep(args.delay)
