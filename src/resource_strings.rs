// Paths
pub(super) const PATH_RESOURCE_DIRECTORY_NAME: &str = "resources";
pub(super) const PATH_ROMS_DIRECTORY_NAME: &str = "roms";
pub(super) const PATH_OPTIONS_DIRECTORY_NAME: &str = "options";

// Window titles
pub(super) const TITLE_APP_WINDOW: &str = "Chipolata: CHIP-8 emulator";
pub(super) const TITLE_LOAD_PROGRAM_WINDOW: &str = "Locate program ROM to load";
pub(super) const TITLE_LOAD_OPTIONS_WINDOW: &str = "Locate options file to load";
pub(super) const TITLE_SAVE_OPTIONS_WINDOW: &str = "Locate options file to save";
pub(super) const TITLE_OPTIONS_WINDOW: &str = "Emulation Options";
pub(super) const TITLE_LOAD_OPTIONS_ERROR_WINDOW: &str = "Error";
pub(super) const TITLE_SAVE_OPTIONS_ERROR_WINDOW: &str = "Error";

// Error messages
pub(super) const ERROR_LOAD_OPTIONS: &str = "Could not load options from file";
pub(super) const ERROR_SAVE_OPTIONS: &str = "Could not save options to file";

// Widget captions
pub(super) const CAPTION_BUTTON_LOAD_PROGRAM: &str = "Load Program";
pub(super) const CAPTION_BUTTON_OPTIONS: &str = "Options";
pub(super) const CAPTION_BUTTON_RUN: &str = "▶";
pub(super) const CAPTION_BUTTON_PAUSE: &str = "⏸";
pub(super) const CAPTION_BUTTON_RESTART: &str = "⏮";
pub(super) const CAPTION_BUTTON_STOP: &str = "⏹";
pub(super) const CAPTION_BUTTON_LOAD_OPTIONS: &str = "Load From File";
pub(super) const CAPTION_BUTTON_SAVE_OPTIONS: &str = "Save To File";
pub(super) const CAPTION_BUTTON_OK: &str = "OK";
pub(super) const CAPTION_BUTTON_CANCEL: &str = "Cancel";
pub(super) const CAPTION_PROCESSOR_SPEED_SUFFIX: &str = "hz";
pub(super) const CAPTION_LABEL_PROCESSOR_SPEED: &str = "CPU cycles/s (target): ";
pub(super) const CAPTION_LABEL_PROGRAM_ADDRESS: &str = "Program start address (hex): ";
pub(super) const CAPTION_LABEL_FONT_ADDRESS: &str = "Font start address (hex): ";
pub(super) const CAPTION_LABEL_FOREGROUND_COLOUR: &str = "Foreground colour: ";
pub(super) const CAPTION_LABEL_BACKGROUND_COLOUR: &str = "Background colour: ";
pub(super) const CAPTION_LABEL_EXECUTION_STATUS: &str = "Execution status: ";
pub(super) const CAPTION_LABEL_ERROR: &str = "ERROR: ";
pub(super) const CAPTION_LABEL_MODE_SPECIFIC_OPTIONS: &str = "Mode-specific options: ";
pub(super) const CAPTION_LABEL_CYCLES_PER_SECOND: &str = "CPU cycles/s (actual): ";
pub(super) const CAPTION_LABEL_GETTING_STARTED_1: &str =
    "Welcome to Chipolata, a CHIP-8 interpreter with compatibility options to enable
emulation of key historic interpreters: CHIP-8, CHIP-48 and SUPER-CHIP 1.1.";
pub(super) const CAPTION_LABEL_GETTING_STARTED_2: &str =
    "To begin, click the 'Load Program' button above and specify a CHIP-8 ROM file
for Chipolata to run.  Before execution begins, you will be prompted to specify
emulation options (or load a set of pre-configured options from a file), however
for most ROMs the default options should be sufficient. Emulation options can be
reconfigured at a later point by clicking the 'Options' button.";
pub(super) const CAPTION_LABEL_GETTING_STARTED_3: &str =
    "While a program is running, execution can be paused and resumed using the
▶/⏸ button at the bottom left of the window, and the program can be reset and
restarted by clicking ⏮.  The ⏹ button resets the emulator entirely.";
pub(super) const CAPTION_LABEL_GETTING_STARTED_4: &str =
    "Overall emulation speed can be controlled using the 'CPU cycles/s (target)'
slider, which sets the number of CHIP-8 instructions Chipolata will aim to execute
every second.  The actual speed achieved - along with current execution status -
is displayed at the bottom right of the window.  This speed may be adjusted in
real-time while a program is running without interrupting execution; changing
any other emulation options will trigger a program restart.";
pub(super) const CAPTION_LABEL_GETTING_STARTED_5: &str =
    "CHIP-8/SUPER-CHIP can only display two colours; these can be configured using
the 'Foreground colour' and 'Background colour' colour pickers at the top right
of the window.";
pub(super) const CAPTION_LABEL_GETTING_STARTED_6: &str =
    "If an error occurs during execution, Chipolata will alert you to this in bold, red
text above the status bar at the bottom of the window.  In most cases you can click
the ⏮ button to recover and restart the program; worst case you may choose to
load a different ROM file instead.";
pub(super) const CAPTION_LABEL_KEYBOARD_CONTROLS_1: &str =
    "The early computers for which CHIP-8 was designed had hexadecimal
keypads for user input, with 16 keys in a 4x4 grid:";
pub(super) const CAPTION_LABEL_KEYBOARD_CONTROLS_2: &str =
    "For convenience when using a modern QWERTY keyboard, Chipolata maps
the original CHIP-8 buttons to your keyboard as follows:";
pub(super) const CAPTION_LABEL_ABOUT_1: &str = "This version of the software: ";
pub(super) const CAPTION_LABEL_ABOUT_2: &str =
    "Chipolata is created by Jon Axon. Source code and latest release on Github:";
pub(super) const CAPTION_RADIO_CHIP8: &str = "CHIP-8";
pub(super) const CAPTION_RADIO_CHIP48: &str = "CHIP-48";
pub(super) const CAPTION_RADIO_SCHIP: &str = "SUPER-CHIP 1.1";
pub(super) const CAPTION_CHECKBOX_MEMORY_LIMIT: &str = "2KB memory limit";
pub(super) const CAPTION_CHECKBOX_CYCLE_TIMING: &str = "Variable cycle timing";
pub(super) const CAPTION_CHECKBOX_OCTO_COMPATIBILITY: &str = "Octo compatibility mode";
pub(super) const CAPTION_HEADING_EMULATION_MODE: &str = "Emulation Mode";
pub(super) const CAPTION_HEADING_OPTIONS_COMMON: &str = "Common Settings";
pub(super) const CAPTION_HEADING_OPTIONS_LOAD_SAVE: &str = "Load/Save Options";
pub(super) const CAPTION_HEADING_GETTING_STARTED: &str = "Getting Started";
pub(super) const CAPTION_HEADING_KEYBOARD_CONTROLS: &str = "Keyboard Controls";
pub(super) const CAPTION_HEADING_ABOUT: &str = "About";

// File dialog filters
pub(super) const FILTER_CHIP8: &str = "CHIP-8";
pub(super) const FILTER_JSON: &str = "JSON";
pub(super) const FILTER_ALL: &str = "All";

// Ui element IDs
pub(super) const ID_TOP_PANEL: &str = "top_panel";
pub(super) const ID_BOTTOM_PANEL: &str = "bottom_panel";
pub(super) const ID_OPTIONS_MODAL: &str = "options_modal";
pub(super) const ID_OPTIONS_MODAL_GRID: &str = "options_modal_grid";
pub(super) const ID_KEYBOARD_CONTROLS_GRID_1: &str = "keyboard_controls_grid_1";
pub(super) const ID_KEYBOARD_CONTROLS_GRID_2: &str = "keyboard_controls_grid_2";

// Links
pub(super) const LINK_GITHUB: &str = "https://github.com/jon-axon/chipolata";

// Tooltips
pub(super) const TOOLTIP_BUTTON_LOAD_PROGRAM: &str = "Load and run a CHIP-8 ROM file from disk";
pub(super) const TOOLTIP_BUTTON_OPTIONS: &str =
    "Configure Chipolata emulation options and compatibility settings";
pub(super) const TOOLTIP_BUTTON_OPTIONS_DISABLED: &str =
    "Configure Chipolata emulation options and compatibility settings.  Disabled when no program ROM is loaded";
pub(super) const TOOLTIP_BUTTON_RUN: &str = "Resume execution of the current program";
pub(super) const TOOLTIP_BUTTON_RUN_DISABLED: &str =
    "Resume execution of the current program.  Disabled if no program ROM is loaded, or if execution has crashed";
pub(super) const TOOLTIP_BUTTON_PAUSE: &str = "Pause execution of the current program";
pub(super) const TOOLTIP_BUTTON_RESTART: &str =
    "Reset and restart the currently loaded program ROM";
pub(super) const TOOLTIP_BUTTON_RESTART_DISABLED: &str =
    "Reset and restart the currently loaded program ROM.  Disabled when no program ROM is loaded";
pub(super) const TOOLTIP_BUTTON_STOP: &str = "Stop and reset Chipolata";
pub(super) const TOOLTIP_BUTTON_STOP_DISABLED: &str =
    "Stop and reset Chipolata.  Disabled when no program is running";
pub(super) const TOOLTIP_BUTTON_LOAD_OPTIONS: &str =
    "Load pre-configured options settings file from disk";
pub(super) const TOOLTIP_BUTTON_SAVE_OPTIONS: &str =
    "Save current options to disk as a settings file";
pub(super) const TOOLTIP_COLOUR_PICKER_FOREGROUND: &str =
    "Change the colour used to render 'on' pixels";
pub(super) const TOOLTIP_COLOUR_PICKER_BACKGROUND: &str =
    "Change the colour used to render 'off' pixels";
pub(super) const TOOLTIP_SLIDER_PROCESSOR_SPEED: &str =
    "Drag or type to set the target processor speed (cycles per second)";
pub(super) const TOOLTIP_SLIDER_PROCESSOR_SPEED_DISABLED: &str =
    "Drag or type to set the target processor speed (cycles per second).  Disabled when emulating CHIP-8 variable cycle timing";
pub(super) const TOOLTIP_SLIDER_PROGRAM_ADDRESS: &str =
    "Drag or type to set the memory address into which the program ROM will start to be loaded";
pub(super) const TOOLTIP_SLIDER_FONT_ADDRESS: &str =
    "Drag or type to set the memory address into which the CHIP-8 font will start to be loaded";
pub(super) const TOOLTIP_SELECTABLE_CHIP8: &str =
    "Emulate the classic COSMAC VIP CHIP-8 interpreter";
pub(super) const TOOLTIP_SELECTABLE_CHIP48: &str =
    "Emulate the reimplementation of CHIP-8 for the HP48 graphing calculators";
pub(super) const TOOLTIP_SELECTABLE_SUPERCHIP: &str =
    "Emulate version 1.1 of the enhanced SUPER-CHIP interpreter";
pub(super) const TOOLTIP_BUTTON_OPTIONS_OK: &str =
    "Apply the selected options.  If a program is already running, this will cause it to restart";
pub(super) const TOOLTIP_BUTTON_OPTIONS_CANCEL: &str = "Discard any options changes";
pub(super) const TOOLTIP_CHECKBOX_MEMORY_LIMIT: &str = "Emulate a COSMAC VIP with only 2KB of memory rather than 4KB.  WARNING: likely to crash most ROMs!";
pub(super) const TOOLTIP_CHECKBOX_VARIABLE_CYCLE_TIMING: &str = "Rather than using fixed cycle lengths for all opcodes, emulate original COSMAC VIP opcode timings and processor speed.  Experimental feature!";
pub(super) const TOOLTIP_CHECKBOX_OCTO_COMPATIBILITY: &str = "Emulate deviations from the original SUPER-CHIP 1.1 specification implemented by the popular Octo interpreter (try enabling this for any problematic SUPER-CHIP ROMs)";
