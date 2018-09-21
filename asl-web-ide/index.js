import * as monaco from 'monaco-editor';
import "./style.css";
import binaryen from "binaryen";

const compiledModule = fetch("asl_lang.wasm").then((r) => r.arrayBuffer()).then((b) => WebAssembly.compile(b));

let decodeUtf8;
if (!global["TextDecoder"]) {
    decodeUtf8 = (data) => {
        var str = '',
            i;

        for (i = 0; i < data.length; i++) {
            var value = data[i];

            if (value < 0x80) {
                str += String.fromCharCode(value);
            } else if (value > 0xBF && value < 0xE0) {
                str += String.fromCharCode((value & 0x1F) << 6 | data[i + 1] & 0x3F);
                i += 1;
            } else if (value > 0xDF && value < 0xF0) {
                str += String.fromCharCode((value & 0x0F) << 12 | (data[i + 1] & 0x3F) << 6 | data[i + 2] & 0x3F);
                i += 2;
            } else {
                var charCode = ((value & 0x07) << 18 | (data[i + 1] & 0x3F) << 12 | (data[i + 2] & 0x3F) << 6 | data[i + 3] & 0x3F) - 0x010000;

                str += String.fromCharCode(charCode >> 10 | 0xD800, charCode & 0x03FF | 0xDC00);
                i += 3;
            }
        }

        return str;
    };
} else {
    const decoder = new TextDecoder("UTF-8");
    decodeUtf8 = (data) => decoder.decode(data);
}

let encodeUtf8;
if (!global["TextEncoder"]) {
    encodeUtf8 = (str) => {
        var utf8 = [];
        for (var i = 0; i < str.length; i++) {
            var charcode = str.charCodeAt(i);
            if (charcode < 0x80) {
                utf8.push(charcode);
            } else if (charcode < 0x800) {
                utf8.push(0xc0 | (charcode >> 6),
                    0x80 | (charcode & 0x3f));
            }
            else if (charcode < 0xd800 || charcode >= 0xe000) {
                utf8.push(0xe0 | (charcode >> 12),
                    0x80 | ((charcode >> 6) & 0x3f),
                    0x80 | (charcode & 0x3f));
            } else {
                i++;
                charcode = 0x10000 + (((charcode & 0x3ff) << 10)
                    | (str.charCodeAt(i) & 0x3ff))
                utf8.push(0xf0 | (charcode >> 18),
                    0x80 | ((charcode >> 12) & 0x3f),
                    0x80 | ((charcode >> 6) & 0x3f),
                    0x80 | (charcode & 0x3f));
            }
        }
        return new Uint8Array(utf8);
    };
} else {
    const encoder = new TextEncoder("UTF-8");
    encodeUtf8 = (str) => encoder.encode(str);
}

self.MonacoEnvironment = {
    getWorkerUrl: function (moduleId, label) {
        return './editor.worker.bundle.js';
    }
}

monaco.languages.register({ id: "wast" });

monaco.languages.setMonarchTokensProvider('wast', {
    // Set defaultToken to invalid to see what you do not tokenize yet
    // defaultToken: 'invalid',
    keywords: [
        "module",
        "table",
        "memory",
        "export",
        "import",
        "func",
        "result",
        "offset",
        "anyfunc",
        "type",
        "data",
        "start",
        "element",
        "global",
        "local",
        "mut",
        "param",
        "result",
        "call",
        "drop",

        "i32.load8_s",
        "i32.load8_u",
        "i32.load16_s",
        "i32.load16_u",
        "i32.load",
        "i64.load8_s",
        "i64.load8_u",
        "i64.load16_s",
        "i64.load16_u",
        "i64.load32_s",
        "i64.load32_u",
        "i64.load",
        "f32.load",
        "f64.load",

        "i32.store8",
        "i32.store16",
        "i32.store",
        "i64.store8",
        "i64.store16",
        "i64.store32",
        "i64.store",
        "f32.store",
        "f64.store",

        "i32.const",
        "i64.const",
        "f32.const",
        "f64.const",

        "i32.add",
        "i32.sub",
        "i32.mul",
        "i32.div_s",
        "i32.div_u",
        "i32.rem_s",
        "i32.rem_u",
        "i32.and",
        "i32.or",
        "i32.xor",
        "i32.shl",
        "i32.shr_u",
        "i32.shr_s",
        "i32.rotl",
        "i32.rotr",
        "i32.eq",
        "i32.ne",
        "i32.lt_s",
        "i32.le_s",
        "i32.lt_u",
        "i32.le_u",
        "i32.gt_s",
        "i32.ge_s",
        "i32.gt_u",
        "i32.ge_u",
        "i32.clz",
        "i32.ctz",
        "i32.popcnt",
        "i32.eqz",

        "i64.add",
        "i64.sub",
        "i64.mul",
        "i64.div_s",
        "i64.div_u",
        "i64.rem_s",
        "i64.rem_u",
        "i64.and",
        "i64.or",
        "i64.xor",
        "i64.shl",
        "i64.shr_u",
        "i64.shr_s",
        "i64.rotl",
        "i64.rotr",
        "i64.eq",
        "i64.ne",
        "i64.lt_s",
        "i64.le_s",
        "i64.lt_u",
        "i64.le_u",
        "i64.gt_s",
        "i64.ge_s",
        "i64.gt_u",
        "i64.ge_u",
        "i64.clz",
        "i64.ctz",
        "i64.popcnt",
        "i64.eqz",

        "f32.add",
        "f32.sub",
        "f32.mul",
        "f32.div",
        "f32.abs",
        "f32.neg",
        "f32.copysign",
        "f32.ceil",
        "f32.floor",
        "f32.trunc",
        "f32.nearest",
        "f32.eq",
        "f32.ne",
        "f32.lt",
        "f32.le",
        "f32.gt",
        "f32.ge",
        "f32.sqrt",
        "f32.min",
        "f32.max",

        "f64.add",
        "f64.sub",
        "f64.mul",
        "f64.div",
        "f64.abs",
        "f64.neg",
        "f64.copysign",
        "f64.ceil",
        "f64.floor",
        "f64.trunc",
        "f64.nearest",
        "f64.eq",
        "f64.ne",
        "f64.lt",
        "f64.le",
        "f64.gt",
        "f64.ge",
        "f64.sqrt",
        "f64.min",
        "f64.max",

        "i32.wrap/i64",
        "i32.trunc_s/f32",
        "i32.trunc_s/f64",
        "i32.trunc_u/f32",
        "i32.trunc_u/f64",
        "i32.reinterpret/f32",
        "i64.extend_s/i32",
        "i64.extend_u/i32",
        "i64.trunc_s/f32",
        "i64.trunc_s/f64",
        "i64.trunc_u/f32",
        "i64.trunc_u/f64",
        "i64.reinterpret/f64",
        "f32.demote/f64",
        "f32.convert_s/i32",
        "f32.convert_s/i64",
        "f32.convert_u/i32",
        "f32.convert_u/i64",
        "f32.reinterpret/i32",
        "f64.promote/f32",
        "f64.convert_s/i32",
        "f64.convert_s/i64",
        "f64.convert_u/i32",
        "f64.convert_u/i64",
        "f64.reinterpret/i64",

        "get_local",
        "set_local",
        "tee_local",
        "get_global",
        "set_global",

        "current_memory",
        "grow_memory"
    ],

    typeKeywords: [
        "i32",
        "i64",
        "f32",
        "f64",
        "anyfunc"
    ],

    operators: [
    ],

    brackets: [
        ["(", ")", "bracket.parenthesis"],
        ["{", "}", "bracket.curly"],
        ["[", "]", "bracket.square"]
    ],

    // we include these common regular expressions
    symbols: /[=><!~?:&|+\-*\/\^%]+/,

    // C# style strings
    escapes: /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,

    // The main tokenizer for our languages
    tokenizer: {
        root: [
            // identifiers and keywords
            [/[a-zA-Z_$][\w$\.]*/, {
                cases: {
                    "@keywords": "keyword",
                    "@typeKeywords": "type",
                    "@default": "type.identifier"
                }
            }],

            // numbers
            [/\d+/, "number"],

            // strings
            [/"/, { token: "string.quote", bracket: "@open", next: "@string" }],

            [/[{}()\[\]]/, "@brackets"]
        ],

        comment: [
            [/[^\/*]+/, "comment"],
            [/\/\*/, "comment", "@push"],    // nested comment
            ["\\*/", "comment", "@pop"],
            [/[\/*]/, "comment"]
        ],

        string: [
            [/[^\\"]+/, "string"],
            [/@escapes/, "string.escape"],
            [/\\./, "string.escape.invalid"],
            [/"/, { token: "string.quote", bracket: "@close", next: "@pop" }]
        ],

        whitespace: [
            [/[ \t\r\n]+/, "white"],
            [/\/\*/, "comment", "@comment"],
            [/\/\/.*$/, "comment"],
        ],
    },
});

monaco.languages.setLanguageConfiguration("wast", {
    brackets: [
        ["(", ")"],
        ["{", "}"],
        ["[", "]"],
    ],
    comments: {
        blockComment: ["/*", "*/"],
        lineComment: "//",
    },
});

monaco.languages.register({ id: "asl" });

monaco.languages.setLanguageConfiguration("asl", {
    brackets: [
        ["(", ")"],
        ["{", "}"],
        ["[", "]"],
    ],
    comments: {
        blockComment: ["/*", "*/"],
        lineComment: "//",
    },
});

monaco.languages.setMonarchTokensProvider('asl', {
    // Set defaultToken to invalid to see what you do not tokenize yet
    // defaultToken: 'invalid',

    keywords: [
        "if", "else", "for", "match", "while", "loop", "let",
        "in", "as"
    ],

    functionFollows: ["fn"],

    booleans: [
        "true", "false"
    ],

    actions: [
        "state", "start", "split", "reset", "isLoading", "gameTime"
    ],

    typeKeywords: [
        "bool", "unit", "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "f32", "f64",
        "{int}", "{float}", "{number}", "{bits}",
    ],

    operators: [
        '=', '>', '<', '!', '~', '?', '==', '<=', '>=', '!=',
        '&&', '||', '++', '--', '+', '-', '*', '/', '&', '|', '^', '%',
        '<<', '>>', '+=', '-=', '*=', '/=', '&=', '|=', '^=',
        '%=', '<<=', '>>=', '..', "..=", "=>", "->"
    ],

    // we include these common regular expressions
    symbols: /[=><!~?:&|\.+\-*\/\^%]+/,

    // C# style strings
    escapes: /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,

    // The main tokenizer for our languages
    tokenizer: {
        root: [
            // identifiers and keywords
            [/[a-z_$][\w$]*/, {
                cases: {
                    '@booleans': 'constant',
                    '@functionFollows': { token: 'keyword', next: "@functionName" },
                    '@typeKeywords': 'storage.type',
                    '@actions': 'entity.name.function',
                    '@keywords': 'keyword',
                    '@default': 'identifier',
                }
            }],

            // whitespace
            { include: '@whitespace' },

            // delimiters and operators
            [/[{}()\[\]]/, '@brackets'],
            // [/[<>](?!@symbols)/, '@brackets'],
            [/@symbols/, {
                cases: {
                    '@operators': 'operator',
                    '@default': ''
                }
            }],

            // numbers
            [/\d*\.\d+([eE][\-+]?\d+)?/, 'number.float'],
            [/0[xX][0-9a-fA-F]+/, 'number.hex'],
            [/\d+/, 'number'],

            // delimiter: after number because of .\d floats
            [/[;,.]/, 'delimiter'],

            // strings
            [/"([^"\\]|\\.)*$/, 'string.invalid'],  // non-teminated string
            [/"/, { token: 'string.quote', bracket: '@open', next: '@string' }],

            // characters
            [/'[^\\']'/, 'string'],
            [/(')(@escapes)(')/, ['string', 'string.escape', 'string']],
            [/'/, 'string.invalid']
        ],

        comment: [
            [/[^\/*]+/, 'comment'],
            [/\/\*/, 'comment', '@push'],    // nested comment
            ["\\*/", 'comment', '@pop'],
            [/[\/*]/, 'comment']
        ],

        string: [
            [/[^\\"]+/, 'string'],
            [/@escapes/, 'string.escape'],
            [/\\./, 'string.escape.invalid'],
            [/"/, { token: 'string.quote', bracket: '@close', next: '@pop' }]
        ],

        whitespace: [
            [/[ \t\r\n]+/, 'white'],
            [/\/\*/, 'comment', '@comment'],
            [/\/\/.*$/, 'comment'],
        ],

        functionName: [
            { include: '@whitespace' },
            [/[a-zA-Z_][\w]*/, 'entity.name.function'],
            ['', '', '@pop'],
        ],
    },
});

monaco.editor.defineTheme('asl', {
    base: 'vs-dark',
    inherit: true,
    rules: [
        { token: 'keyword', foreground: 'F92672' },
        { token: 'operator', foreground: 'F92672' },
        { token: 'string', foreground: "FFEE99" },
        { token: "storage.type", foreground: "66D9EF", fontStyle: 'italic' },
        { token: "type", foreground: "66D9EF", fontStyle: 'italic' },
        { token: "number", foreground: "A477F6" },
        { token: "constant", foreground: "A477F6" },
        { token: "number.float", foreground: "A477F6" },
        { token: "number.hex", foreground: "A477F6" },
        { token: "entity.name.function", foreground: "A6E22E" },
        { token: "selection", foreground: "403d3d" }
    ],
    colors: {
        'editor.selectionBackground': '#3d3d3d',
        'editor.lineHighlightBackground': "#3D3D3D55",
    },
});

monaco.languages.registerCompletionItemProvider('asl', {
    provideCompletionItems: () => {
        return [
            {
                label: 'state',
                kind: monaco.languages.CompletionItemKind.Keyword,
                insertText: {
                    value: `state("$\{1:game.exe}") {
    $0
}`
                }
            },
            {
                label: 'fn ',
                kind: monaco.languages.CompletionItemKind.Keyword,
                insertText: {
                    value: `fn $\{1:name}($\{2:params}) $\{3:-> }$\{4:type} {
    $0
}`
                }
            },
            {
                label: 'start',
                kind: monaco.languages.CompletionItemKind.Keyword,
                insertText: {
                    value: `start {
    $\{0:false\}
}`
                }
            },
            {
                label: 'split',
                kind: monaco.languages.CompletionItemKind.Keyword,
                insertText: {
                    value: `split {
    $\{0:false\}
}`
                }
            },
            {
                label: 'reset',
                kind: monaco.languages.CompletionItemKind.Keyword,
                insertText: {
                    value: `reset {
    $\{0:false\}
}`
                }
            },
            {
                label: 'isLoading',
                kind: monaco.languages.CompletionItemKind.Keyword,
                insertText: {
                    value: `isLoading {
    $\{0:false\}
}`
                }
            },
            {
                label: 'gameTime',
                kind: monaco.languages.CompletionItemKind.Keyword,
                insertText: {
                    value: `gameTime {
    $\{0:0.0\}
}`
                }
            },
            {
                label: 'Pointer Path',
                kind: monaco.languages.CompletionItemKind.Snippet,
                insertText: {
                    value: "${1:name}: ${2:type} = \"${3:module}\", ${4:0x0};",
                },
                documentation: "Pointer Path",
            },
            {
                label: 'current',
                kind: monaco.languages.CompletionItemKind.Keyword,
                insertText: {
                    value: `current.$\{0:field}`
                }
            },
            {
                label: 'old',
                kind: monaco.languages.CompletionItemKind.Keyword,
                insertText: {
                    value: `old.$\{0:field}`
                }
            },
            {
                label: 'for',
                kind: monaco.languages.CompletionItemKind.Keyword,
                insertText: {
                    value: `for $\{1:index\} in $\{2:from\}..$\{3:to\} {
    $\{0\}
};`
                }
            },
            {
                label: 'while',
                kind: monaco.languages.CompletionItemKind.Keyword,
                insertText: {
                    value: `while $\{1:condition\} {
    $\{0\}
};`
                }
            },
        ]
    }
});

monaco.languages.registerHoverProvider('asl', {
    provideHover: async function (model, position) {
        const result = await hover(position.lineNumber, position.column);
        if (result) {
            const { span, ty } = result;
            return {
                range: new monaco.Range(span.lineFrom, span.columnFrom, span.lineTo, span.columnTo),
                contents: [
                    { value: "```asl\n" + ty + "\n```" },
                ],
            }
        }
    }
});

monaco.languages.registerDefinitionProvider('asl', {
    provideDefinition: async function (model, position) {
        const span = await definition(position.lineNumber, position.column);
        if (span) {
            return {
                range: new monaco.Range(span.lineFrom, span.columnFrom, span.lineTo, span.columnTo),
                uri: model.uri,
            };
        }
    }
});

monaco.languages.registerRenameProvider('asl', {
    provideRenameEdits: async function (model, position, newName) {
        const spans = await findAllReferences(position.lineNumber, position.column);
        if (spans) {
            return {
                edits: [
                    {
                        edits: spans.map(({ lineFrom, columnFrom, lineTo, columnTo }) => {
                            return {
                                range: new monaco.Range(lineFrom, columnFrom, lineTo, columnTo),
                                text: newName,
                            }
                        }),
                        resource: model.uri,
                    },
                ],
            }
        }
    }
});

monaco.languages.registerDocumentHighlightProvider('asl', {
    provideDocumentHighlights: async function (model, position) {
        const spans = await findAllReferences(position.lineNumber, position.column);
        if (spans) {
            return spans.map(({ lineFrom, columnFrom, lineTo, columnTo }) => {
                return {
                    range: new monaco.Range(lineFrom, columnFrom, lineTo, columnTo),
                    kind: monaco.languages.DocumentHighlightKind.Text,
                }
            });
        }
    }
});

monaco.editor.create(document.getElementById('aslContainer'), {
    value: `state("game.exe") {
}

start {
    false
}

split {
    true
}`,
    language: 'asl',
    theme: "asl",
    automaticLayout: true,
    autoIndent: true,
    autoClosingBrackets: true,
    matchBrackets: true,
});

monaco.editor.create(document.getElementById('wastContainer'), {
    value: ``,
    language: 'wast',
    theme: "asl",
    automaticLayout: true,
    autoIndent: true,
    autoClosingBrackets: true,
    matchBrackets: true,
    readOnly: true,
});

monaco.editor.getModels()[1].updateOptions({ tabSize: 1 });

async function prepareInstance() {
    const wasm = await WebAssembly.instantiate(await compiledModule, {
        env: {},
    });

    function allocString(str) {
        const stringBuffer = encodeUtf8(str);
        const len = stringBuffer.length + 1;
        const ptr = wasm.exports.alloc(len);
        const slice = new Uint8Array(wasm.exports.memory.buffer, ptr, len);

        slice.set(stringBuffer);
        slice[len - 1] = 0;

        return { ptr, len };
    }

    const str = allocString(monaco.editor.getModels()[0].getValue());
    return { wasm: wasm, src: str };
}

function getSlicePtrLen(wasm, ptr, len) {
    const memory = new Uint8Array(wasm.exports.memory.buffer);
    const slice = memory.slice(ptr, ptr + len);
    return slice;
}

function getSlice(wasm, bufPtr) {
    const ptr = wasm.exports.Buf_as_ptr(bufPtr);
    const len = wasm.exports.Buf_len(bufPtr);
    return getSlicePtrLen(wasm, ptr, len);
}

function decodeString(wasm, bufPtr) {
    const slice = getSlice(wasm, bufPtr);
    return decodeUtf8(slice);
}

function decodeStringPtrLen(wasm, ptr, len) {
    const slice = getSlicePtrLen(wasm, ptr, len);
    return decodeUtf8(slice);
}

function decodeSpan(wasm, spanPtr) {
    const lineFrom = wasm.exports.Span_line_from(spanPtr);
    const columnFrom = wasm.exports.Span_column_from(spanPtr);
    const lineTo = wasm.exports.Span_line_to(spanPtr);
    const columnTo = wasm.exports.Span_column_to(spanPtr);
    return { lineFrom, columnFrom, lineTo, columnTo };
}

async function baseCompile() {
    const { wasm, src } = await prepareInstance();
    const result = wasm.exports.ASL_compile(src.ptr);
    if (wasm.exports.Result_is_ok(result)) {
        const compiled = wasm.exports.Result_ok(result);
        const compiledASL = getSlice(wasm, compiled);
        return { isOk: true, val: compiledASL };
    } else {
        const err = wasm.exports.Result_error(result);
        const msgPtr = wasm.exports.Error_msg_ptr(err);
        const msgLen = wasm.exports.Error_msg_len(err);
        const spanPtr = wasm.exports.Error_span(err);
        let span = null;
        if (spanPtr != 0) {
            span = decodeSpan(wasm, spanPtr);
        }
        const msg = decodeStringPtrLen(wasm, msgPtr, msgLen);
        return { isOk: false, err: { msg, span } };
    }
}

async function fullCompile() {
    const result = await baseCompile();
    if (result.isOk) {
        const compiledASL = result.val;
        const binaryenModule = binaryen.readBinary(compiledASL);
        binaryenModule.optimize();
        const wast = binaryenModule.emitText();
        monaco.editor.getModels()[1].setValue(wast);
        binaryenModule.dispose();
    }
    return result;
}

async function hover(line, column) {
    const { wasm, src } = await prepareInstance();
    const result = wasm.exports.ASL_hover(src.ptr, line, column);
    if (wasm.exports.Result_is_ok(result)) {
        const hover = wasm.exports.Result_ok(result);
        if (hover != 0) {
            const tyBuf = wasm.exports.Hover_ty(hover);
            const span = wasm.exports.Hover_span(hover);
            const ty = decodeString(wasm, tyBuf);
            return { ty, span: decodeSpan(wasm, span) };
        }
    }
}

async function definition(line, column) {
    const { wasm, src } = await prepareInstance();
    const result = wasm.exports.ASL_go_to_definition(src.ptr, line, column);
    if (wasm.exports.Result_is_ok(result)) {
        const span = wasm.exports.Result_ok(result);
        if (span != 0) {
            return decodeSpan(wasm, span);
        }
    }
}

async function findAllReferences(line, column) {
    const { wasm, src } = await prepareInstance();
    const result = wasm.exports.ASL_find_all_references(src.ptr, line, column);
    if (wasm.exports.Result_is_ok(result)) {
        const spans = wasm.exports.Result_ok(result);
        if (spans != 0) {
            let decodedSpans = [];
            let index = 0;
            while (true) {
                const span = wasm.exports.Spans_get(spans, index);
                if (span == 0) {
                    break;
                }
                decodedSpans.push(decodeSpan(wasm, span));
                index += 1;
            }
            return decodedSpans;
        }
    }
}

const compileButton = document.getElementById('compile');
compileButton.onclick = () => fullCompile();

const body = document.getElementsByTagName("body")[0];

async function validate() {
    let success = true;
    monaco.editor.setModelMarkers(monaco.editor.getModels()[0], "i made this", []);
    try {
        const result = await fullCompile();
        if (!result.isOk) {
            success = false;
            const { msg, span } = result.err;
            compileButton.textContent = `Error: ${msg}`;
            if (span) {
                monaco.editor.setModelMarkers(monaco.editor.getModels()[0], "i made this", [
                    {
                        startLineNumber: span.lineFrom,
                        endLineNumber: span.lineTo,
                        startColumn: span.columnFrom,
                        endColumn: span.columnTo,
                        severity: monaco.MarkerSeverity.Error,
                        message: msg,
                    }
                ])
            }
        } else {
            compileButton.textContent = "Download";
        }
    } catch (e) {
        success = false;
        compileButton.textContent = "Internal Compiler Error";
    }
    body.style.backgroundColor = success ? "hsl(120, 100%, 20%)" : "hsl(0, 100%, 20%)";
}

monaco.editor.getModels()[0].onDidChangeContent(() => {
    setTimeout(validate, 0);
});

validate();
