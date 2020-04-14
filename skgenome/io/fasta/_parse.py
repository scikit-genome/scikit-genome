import lark


class _Transformer(lark.Transformer):
    @staticmethod
    def description(token):
        if token:
            return token[0].lstrip(">").strip()
        else:
            return None

    @staticmethod
    def section(token):
        description, sequence = token

        return sequence, description

    @staticmethod
    def sections(token):
        return token

    @staticmethod
    def sequence(token):
        if token:
            return token[0].strip()
        else:
            return None

    @staticmethod
    def sequences(token):
        if token:
            return "".join(token)


def parse(pathname):
    parser_options = {
        "parser": "lalr",
        "rel_to": __file__,
        "start": "unit",
        "transformer": _Transformer(),
    }

    parser = lark.Lark.open("grammar.lark", **parser_options)

    with open(pathname, "r") as fp:
        data = fp.read()

    return parser.parse(data)
