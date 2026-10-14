import {
  CLAUDE_CODE_COMMAND_FALLBACK_CHARACTER_LOGICAL_WIDTH,
  CLAUDE_CODE_EXPANDED_LOGICAL_PADDING,
  CLAUDE_CODE_PERMISSION_ALERT_LOGICAL_WIDTH,
  CLAUDE_CODE_PERMISSION_COMMAND_SCROLLBAR_LOGICAL_WIDTH,
  CLAUDE_CODE_PERMISSION_DESCRIPTION_LOGICAL_PADDING_INLINE,
} from "../../frontendConstants";

// The alert is one tile of its own width, whatever the dashboard's width.
const COMMAND_TEXT_LOGICAL_WIDTH =
  CLAUDE_CODE_PERMISSION_ALERT_LOGICAL_WIDTH -
  2 * CLAUDE_CODE_EXPANDED_LOGICAL_PADDING -
  2 * CLAUDE_CODE_PERMISSION_DESCRIPTION_LOGICAL_PADDING_INLINE -
  CLAUDE_CODE_PERMISSION_COMMAND_SCROLLBAR_LOGICAL_WIDTH;
// Measured over many characters, so the font's fractional width isn't rounded away.
const MEASURED_SAMPLE_TEXT = "M".repeat(100);
const PRINTABLE_ASCII_PATTERN = /^[\x20-\x7e]$/;

let measuredCharacterWidth: number | null = null;

// How many rows a command takes in the alert's box, which wraps at any character: each line at
// least one. Printable ASCII is one cell of the monospace font; anything else counts as two
// (wide glyphs, other fonts), so the estimate errs high. It only sizes the box: whether Allow
// may be pressed is decided by the box's real scroll position.
export function estimateClaudeCodeCommandRowCount(commandText: string): number {
  const charactersPerRow = Math.max(1, Math.floor(COMMAND_TEXT_LOGICAL_WIDTH / measureCommandCharacterWidth()));
  return commandText.split("\n").reduce((rowCount, commandLine) => {
    const lineCells = Array.from(commandLine).reduce(
      (cellCount, lineCharacter) => cellCount + (PRINTABLE_ASCII_PATTERN.test(lineCharacter) ? 1 : 2),
      0,
    );
    return rowCount + Math.max(1, Math.ceil(lineCells / charactersPerRow));
  }, 0);
}

// Once per page: the command font is fixed, and a monospace font gives every ASCII character
// the same width. Reads the font from the design tokens, so it is the one the box uses.
function measureCommandCharacterWidth(): number {
  if (measuredCharacterWidth === null) {
    const rootStyle = getComputedStyle(document.documentElement);
    const commandFontSize = rootStyle.getPropertyValue("--claude-code-command-font-size").trim();
    const commandFontFamily = rootStyle.getPropertyValue("--claude-code-command-font-family").trim();
    const measuringContext = document.createElement("canvas").getContext("2d");
    if (!measuringContext) {
      return CLAUDE_CODE_COMMAND_FALLBACK_CHARACTER_LOGICAL_WIDTH;
    }
    measuringContext.font = `${commandFontSize} ${commandFontFamily}`;
    measuredCharacterWidth = measuringContext.measureText(MEASURED_SAMPLE_TEXT).width / MEASURED_SAMPLE_TEXT.length;
  }
  return measuredCharacterWidth;
}
