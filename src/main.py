import pickle as pk
import sys

APP_NAME = "APP NAME"
COMMANDS = ["enc", "dec", "help"]

"""
return code meanings:
0 = normal
-1 = invalid command
"""


def main() -> int:
    argc: int = len(sys.argv)
    command: str
    arg_word: str

    if argc == 1:
        return command_error("No command given.")

    arg_word = "was" if argc == 2 else "were"

    if sys.argv[1] in COMMANDS:
        command = sys.argv[1]
    else:
        return command_error(f'Unkown command "{sys.argv[1]}".')

    if command == "help":
        if argc != 2:
            return command_error(f"Expected 1 argument but {argc - 1} {arg_word} given.")
        else:
            print(f"Usage: {APP_NAME} [COMMAND] [ARGUMENTS]")
            print(
                "Commands:\n"
                + "    help                             Help about commands.\n"
                + "    enc [SRC PATH] [OUT PATH]        Encodes the file given in SRC PATH and outputs the result to OUT PATH.\n"
                + "    dec [SRC PATH] [OUT PATH]        Decodes the file given in SRC PATH and outputs the result to OUT PATH."
            )
            return 0

    if command == "enc":
        if argc != 4:
            return command_error(f"Expected 3 arguments but {argc - 1} {arg_word} given.")
        else:
            print("TODO encode")
            # encode code
            return 0

    if command == "dec":
        if argc != 4:
            return command_error(f"Expected 3 arguments but {argc - 1} {arg_word} given.")
        else:
            print("TODO decode")
            # decode code
            return 0

    assert False  # should be unreachable


def command_error(error_message: str) -> int:
    print(f"Error: {error_message} (Try {APP_NAME} help)")
    return -1


if __name__ == "__main__":
    raise SystemExit(main()) # returns exit code, allows communication of failure