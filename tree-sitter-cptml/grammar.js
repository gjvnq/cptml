module.exports = grammar({
    name: 'CPTML',

    // Todo: code blocks
    // Todo: use ! at the end for macros, e.g. {my-fancy-macro!; my content}

    rules: {
        source_file: $ => repeat($._node),
        text: $ => choice(/[^{}<|>]+/, /\\[{}<|>]/),
        _node: $ => choice($.comment, $.curly_tag, $.pointy_tag_head, $.pointy_tag_tail, $.value_nodes, $.text),
        comment_text: $ => choice(
            token.immediate(/[^{}%]+/),
            token.immediate(/%[^}]/),
            token.immediate(/[^{]%/)),
        comment: $ => seq('{%', repeat(choice($.comment_text, $.comment)), '%}'),
        pointy_tag_head: $ => seq('<', optional($.view_name), $.identifier, repeat($.attribute), '|'),
        pointy_tag_tail: $ => seq('|', optional($.view_name), $.identifier, '>'),
        curly_tag: $ => choice(
            seq($.curly_tag_head, repeat($.attribute), '}'),
            seq($.curly_tag_head, repeat($.attribute), ';', repeat($._node), '}'),
        ),
        curly_tag_head: $ => seq('{', field('name', $.identifier)),
        attribute: $ => choice(
            prec(2, seq($.identifier, '=', $._value)),
            prec(1, $._value)),
        _value: $ => choice(
            $.value_bool,
            $.value_iri,
            $.value_number_dec,
            $.value_number_hex,
            $.value_number_bin,
            $.value_string,
            $.value_array,
            $.value_dict,
            $.value_nodes
        ),
        value_array: $ => seq('[', optional(
            seq(
                repeat(seq($._value, ',')),
                $._value,
                optional(',')
            )), ']'),
        value_dict: $ => seq('{', optional(
            seq(
                repeat(seq($.dict_entry, ',')),
                $.dict_entry,
                optional(',')
            )), '}'),
        dict_entry: $ => seq(choice($.value_string, $.simple_identifier), ':', $._value),
        _number_bin: $ => token.immediate(/(:?[01]|[01](:?[01]|_)*[01])/),
        _number_dec: $ => token.immediate(/(:?\d|\d(:?\d|_)*\d)/),
        _number_hex: $ => token.immediate(/(:?[0-9ABCDEF]|[0-9ABCDEF](:?[0-9ABCDEF]|_)*[0-9ABCDEF])/i),
        _exponent: $ => token.immediate(/[eE][+-]?\d+/),
        _unicode_escape: $ => seq(token.immediate('\\u{'), token.immediate(/[0-9ABCDEF]{2,6}/i), token.immediate('}')),
        _string_part: $ => choice(
            token.immediate('\\0'),
            token.immediate('\\n'),
            token.immediate('\\r'),
            token.immediate('\\t'),
            token.immediate('\\\\'),
            token.immediate('\\"'),
            token.immediate('\\\''),
            token.immediate(/[^"\\]+/),
            $._unicode_escape,
        ),
        value_nodes: $ => seq('<>', repeat($._node), '</>'),
        value_string: $ => seq('"', repeat($._string_part), '"'),
        value_bool: $ => choice('true', 'false'),
        value_number_hex: $ => seq(token.immediate('0x'), $._number_hex),
        value_number_bin: $ => seq(token.immediate('0b'), $._number_bin),
        value_number_dec: $ => seq(
            optional(token.immediate(/[+-]/)),
            choice(
                prec(7, seq($._number_dec, token.immediate('.'), $._number_dec, $._exponent)),
                prec(6, seq($._number_dec, token.immediate('.'), $._exponent)),
                prec(5, seq($._number_dec, token.immediate('.'), $._number_dec)),
                prec(4, seq($._number_dec, token.immediate('.'))),
                prec(3, seq(token.immediate('.'), $._number_dec, $._exponent)),
                prec(2, seq(token.immediate('.'), $._number_dec)),
                prec(1, seq($._number_dec, $._exponent)),
                prec(0, seq($._number_dec)),
            )),
        value_iri: $ => seq('<', /[^\p{Z}<>]+/, '>'),
        identifier: $ => choice(
            seq(field('ns', $._simple_identifier), ':', field('ln', $._simple_identifier)),
            seq(field('ln', $._simple_identifier)),
            seq(field('ln', $._special_identifier))),
        simple_identifier: $ => $._simple_identifier,
        _simple_identifier: $ => /[^\p{Cc}\p{Cf}\p{Cs}\p{Z}\p{Nd}:{}()<|>="';$!.][^\p{Cc}\p{Cf}\p{Cs}\p{Z}:{}()<|>="';$!]*/,
        _special_identifier: $ => seq('!', $._special_identifier),
        view_name: $ => seq(token.immediate('('), $._simple_identifier, token.immediate(')')),
    }
  });