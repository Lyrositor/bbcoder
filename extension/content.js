"use strict";

const BBCODER_SCRIPT = "https://www.lyros.net/bbcoder.php";
const DEFAULT_BBXML = "<bbxml>\n" +
    "  <templates>\n" +
    "  </templates>\n\n" +
    "  <classes>\n" +
    "  </classes>\n\n" +
    "  <body>\n" +
    "    \n" +
    "  </body>\n" +
    "</bbxml>";
const MESSAGE_EDITOR = "#ctrl_message";
const MODE_BUTTON = "<button type=\"button\" class=\"button mode-btn\"></button>";
const MODE_CONTAINERS = "<div id=\"mode-btn-container\"></div>";
const PREVIEW_BUTTON = ".submitUnit input.PreviewButton";
const SUBMIT_BUTTON = ".submitUnit input[type=submit]";

function setupCodeMirror() {
    CodeMirror.defineMode("bbxml", (config) => {
        return CodeMirror.multiplexingMode(
            CodeMirror.getMode(config, "application/xml"),
            {
                open: /<class .*?>/i,
                close: /<\/class>/i,
                mode: CodeMirror.getMode(config, "text/css"),
            }
        );
    });
}

function switchMode(codeEditor, mode) {
    let previousMode = codeEditor.getOption("mode");

    // Confirm the user wishes to switch mode if switching to or from BBXML
    if ((previousMode === "bbxml" || mode === "bbxml") && codeEditor.getValue().trim()) {
        let ok = confirm("Current editor contents will be lost: are you sure?");
        if (!ok) {
            return;
        }
    }

    // Disable this mode's button and enable all other mode buttons
    $(".mode-btn").prop("disabled", false);
    $("#" + mode + "-mode").prop("disabled", true);

    // Switch the editing mode
    if (mode === "normal") {
        codeEditor.toTextArea();
        $(MESSAGE_EDITOR).focus();
    } else {
        if (previousMode === "normal") {
            codeEditor = setupCodeEditor($(MESSAGE_EDITOR));
        }
        codeEditor.focus();
    }
    codeEditor.setOption("mode", mode);

    // Reset the editor contents if switching to or from BBXML
    if (mode === "bbxml") {
        codeEditor.setValue(DEFAULT_BBXML);
        codeEditor.setCursor(8, 4);
    } else if (previousMode === "bbxml") {
        if (mode === "normal") {
            $(MESSAGE_EDITOR).val("");
        } else {
            codeEditor.setValue("");
        }
    }
}

function setupCodeEditor(messageEditor, defaultValue) {
    // Add the CodeMirror editor
    let codeEditor = CodeMirror.fromTextArea(messageEditor[0], {
        mode: "bbcode",
        autoCloseBrackets: true,
        autoCloseTags: true,
        autofocus: true,  // BUG: Doesn't always focus correctly when loaded
        lineNumbers: true,
        lineWrapping: true,
        matchBrackets: true,
        matchTags: {bothTags: true},
    });
    codeEditor.setOption("extraKeys", {
        Tab: function(cm) {
            let spaces = new Array(cm.getOption("indentUnit") + 1).join(" ");
            cm.replaceSelection(spaces);
        }
    });
    if (defaultValue) {
        codeEditor.setValue(defaultValue);
    }

    // Ensure the textarea is kept updated when in BBCode mode
    codeEditor.on("change", (_) => {
        if (codeEditor.getOption("mode") === "bbcode") {
            codeEditor.save();
        }
    });

    // Bind the mode switching buttons
    $(".mode-btn").off("click");
    $("#normal-mode").click(() => switchMode(codeEditor, "normal"));
    $("#bbcode-mode").click(() => switchMode(codeEditor, "bbcode"));
    $("#bbxml-mode").click(() => switchMode(codeEditor, "bbxml"));

    return codeEditor;
}

function setupModeButtons(messageEditor) {
    let normalModeBtn = $(MODE_BUTTON);
    normalModeBtn.prop("id", "normal-mode");
    normalModeBtn.text("Normal");
    let bbCodeModeBtn = $(MODE_BUTTON);
    bbCodeModeBtn.prop("id", "bbcode-mode");
    bbCodeModeBtn.text("BBCode");
    let bbXmlModeBtn = $(MODE_BUTTON);
    bbXmlModeBtn.prop("id", "bbxml-mode");
    bbXmlModeBtn.text("BBXML");
    let container = $(MODE_CONTAINERS);
    container.append(normalModeBtn);
    container.append(bbCodeModeBtn);
    container.append(bbXmlModeBtn);
    messageEditor.before(container);
    $("#bbcode-mode").prop("disabled", true);
}

function setupFormButtons(codeEditor) {
    // Ensure BBCode is generated when the user presses the Preview and Submit buttons in BBXML mode
    $(PREVIEW_BUTTON + ", " + SUBMIT_BUTTON).each(function () {
        let newButton = $(this).clone(false);
        $(this).after(newButton);
        $(this).hide();

        $(newButton).click((event) => {
            event.preventDefault();
            if (codeEditor.getOption("mode") !== "bbxml") {
                $(this).click();
                return;
            }
            let that = this;
            $.ajax({
                type: "POST",
                url: BBCODER_SCRIPT,
                data: {
                    src: codeEditor.getValue()
                },
                dataType: "json",
                success: (response) => {
                    if (!response) {
                        alert("An unknown error occurred during BBXML conversion.");
                        return;
                    }
                    if (response.status === "success") {
                        $(MESSAGE_EDITOR).val(response.bbcode);
                        $(that).click();
                    } else {
                        alert(response.message);
                    }
                },
                error: (_, textStatus) => {
                    alert("ERROR: " + textStatus);
                }
            });
        });
    });
}

$(document).ready(() => {
    // Check to see if the message editor is present on this page
    let messageEditor = $(MESSAGE_EDITOR);
    if (!messageEditor.length) {
        return;
    }

    // Set up the code editor, using CodeMirror as an editor
    setupCodeMirror();
    setupModeButtons(messageEditor);
    let codeEditor = setupCodeEditor(messageEditor);
    setupFormButtons(codeEditor);
});
