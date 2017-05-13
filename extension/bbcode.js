(function(mod) {
    mod(CodeMirror);
})(function(CodeMirror) {
    "use strict";

    CodeMirror.defineMode("bbcode", function(editorConf, config_) {
        let config = config_;

        // Return variables for tokenizers
        let type, setStyle;

        function inText(stream, state) {
            let ch = stream.next();
            if (ch === "[") {
                type = stream.eat("/") ? "closeTag" : "openTag";
                state.tokenize = inTag;
                return "tag bracket";
            } else {
                stream.eatWhile(/[^\[]/);
                return null;
            }
        }
        inText.isInText = true;

        function inTag(stream, state) {
            let ch = stream.next();
            if (ch === "]") {
                state.tokenize = inText;
                type = "endTag";
                return "tag bracket";
            } else if (ch === "=") {
                type = "equals";
                state.tokenize = inAttribute;
                return null;
            } else {
                stream.match(/^[^=\]]+/);
                return "word";
            }
        }

        function inAttribute(stream, state) {
            this.isInAttribute = true;
            while (!stream.eol()) {
                if (stream.next() === "]") {
                    stream.backUp(1);
                    state.tokenize = inTag;
                    break;
                }
            }
            return "string";
        }

        function Context(state, tagName) {
            this.prev = state.context;
            this.tagName = tagName;
        }

        function popContext(state) {
            if (state.context) {
                state.context = state.context.prev;
            }
        }

        function baseState(type, stream, state) {
            if (type === "openTag") {
                return tagNameState;
            } else if (type === "closeTag") {
                return closeTagNameState;
            } else {
                return baseState;
            }
        }

        function tagNameState(type, stream, state) {
            if (type === "word") {
                state.tagName = stream.current();
                setStyle = "tag";
                return attrEqState;
            } else {
                setStyle = "error";
                return tagNameState;
            }
        }

        function closeTagNameState(type, stream, state) {
            if (type === "word") {
                let tagName = stream.current();
                if ((state.context && state.context.tagName === tagName) || config.matchClosing === false) {
                    setStyle = "tag";
                    return closeState;
                } else {
                    setStyle = "tag error";
                    return closeStateErr;
                }
            } else {
                setStyle = "error";
                return closeStateErr;
            }
        }

        function closeState(type, _stream, state) {
            if (type !== "endTag") {
                setStyle = "error";
                return closeState;
            }
            popContext(state);
            return baseState;
        }

        function closeStateErr(type, stream, state) {
            setStyle = "error";
            return closeState;
        }

        function attrEqState(type, stream, state) {
            if (type === "equals") {
                setStyle = "attribute";
                return attrValueState;
            }
            return attrEndState(type, stream, state);
        }

        function attrEndState(type, stream, state) {
            if (type === "endTag") {
                let tagName = state.tagName;
                state.tagName = null;
                state.context = new Context(state, tagName);
                return baseState;
            }
            setStyle = "error";
            return attrValueState;
        }

        function attrValueState(type, stream, state) {
            if (type !== "string") {
                setStyle = "error";
            }
            return attrEndState;
        }

        return {
            startState: function() {
                return {
                    tokenize: inText,
                    state: baseState,
                    tagName: null,
                    context: null
                };
            },

            token: function(stream, state) {
                if (stream.eatSpace()) {
                    return null;
                }
                type = null;
                let style = state.tokenize(stream, state);
                if ((style || type) && style !== "comment") {
                    setStyle = null;
                    state.state = state.state(type || style, stream, state);
                    if (setStyle) {
                        style = setStyle === "error" ? style + " error" : setStyle;
                    }
                }
                return style;
            },

            electricInput: /\[\/[\s\w:]+\]$/,
            blockCommentStart: "[comment]",
            blockCommentEnd: "[/comment]",

            configuration: "bbcode",
            helperType: "bbcode",

            skipAttribute: function(state) {
                if (state.state === attrValueState) {
                    state.state = attrState
                }
            }
        };
    });

    CodeMirror.defineMIME("text/bbcode", "bbcode");
    CodeMirror.defineMIME("application/bbcode", "bbcode");
});
