import pickle as pk
import sys

APP_NAME = "APP NAME"
COMMANDS = ["enc", "dec", "help"]

def main() -> None:
    argc: int = len(sys.argv)
    command: str
    arg_word: str

    if argc == 1:
        command_error("No command given.")

    arg_word = "was" if argc == 2 else "were"

    if sys.argv[1] in COMMANDS:
        command = sys.argv[1]
    else:
        command_error(f'Unkown command "{sys.argv[1]}".')

    if command == "help":
        if argc != 2:
            command_error(f"Expected 1 argument but {argc - 1} {arg_word} given.")
        else:
            print(f"Usage: {APP_NAME} [COMMAND] [ARGUMENTS]")
            print(
                "Commands:\n"
                + "    help                             Help about commands.\n"
                + "    enc [SRC PATH] [OUT PATH]        Encodes the file given in SRC PATH and outputs the result to OUT PATH.\n"
                + "    dec [SRC PATH] [OUT PATH]        Decodes the file given in SRC PATH and outputs the result to OUT PATH."
            )
            sys.exit()

    if command == "enc":
        if argc != 4:
            return command_error(f"Expected 3 arguments but {argc - 1} {arg_word} given.")
        else:
            print("TODO encode")
            # encode code
            sys.exit()
            

    if command == "dec":
        if argc != 4:
            return command_error(f"Expected 3 arguments but {argc - 1} {arg_word} given.")
        else:
            print("TODO decode")
            # decode code
            sys.exit()
            
    assert False  # should be unreachable


def command_error(error_message: str) -> None:
    print(f"Error: {error_message} (Try {APP_NAME} help)")
    sys.exit()

main()
