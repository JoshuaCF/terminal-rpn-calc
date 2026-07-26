# Terminal RPN Calculator
An RPN calculator which runs from the terminal and exposes all controls through the keyboard.
Colors, layout, and keybindings are configurable.

## Configuration
Configuration of the colors of numbers and separators is possible, as well as the relative orientation
of the stack+entry area and the memory display. All keybinds are configurable, and all commands are configurable.

### config.toml
The config file is created by default at the first location found from the following list:

**Windows**
1. `%APPDATA%\rpn_calc\config.toml`
2. `config.toml` inside the same folder as the executable

**Linux**
1. `$XDG_CONFIG_HOME/rpn_calc/config.toml`
2. `$XDG_HOME/.config/rpn_calc/config.toml`
3. `$HOME/.config/rpn_calc/config.toml`
4. `config.toml` inside the same folder as the executable

**Other**
1. `config.toml` inside the same folder as the executable

If none of these folders can be deterministically located, the program exits and demands the user to specify
a config directory manually as a command line argument.

The user will be prompted to create a new config file if one is not found at the location checked.

### Default Configuration

```toml
[calc]
empty_push_behavior = "Last"
stack_size = 8

[tui.renderer]
stack_alignment = "Left"
memory_alignment = "SplitMiddle"
stack_location = "StackLeft"

[tui.renderer.colors]
decimal_separator = "LightCyan"
exponent_separator = "LightCyan"
number = "Green"
memory_key = "Red"

[tui.parser]
imm_eval_mode = "Numbers"

[tui.parser.immediate_cmds]
- = "Sub"
"*" = "Mul"
"/" = "Div"
"%" = "Mod"
P = "Pow"
enter = "EvalBuf"
"?" = "IntDiv"
F = "CalcStore"
R = "CalcRecall"
delete = "DelChar"
backspace = "DelChar"
C = "Pop"
N = "Neg"
D = "CalcDelete"
S = "Swp"
"+" = "Add"

[tui.parser.string_cmds]
exp = "Exp"
asin = "Asin"
sqr = "Sqr"
atan = "Atan"
quit = "Quit"
sub = "Sub"
mul = "Mul"
root = "Root"
rad = "Rad"
pop = "Pop"
mod = "Mod"
deg = "Deg"
div = "Div"
intdiv = "IntDiv"
neg = "Neg"
swp = "Swp"
cos = "Cos"
pow = "Pow"
add = "Add"
sin = "Sin"
tan = "Tan"
acos = "Acos"
sqrt = "Sqrt"
```

### Options

An understanding of the TOML format is expected. The spec followed by this program is [v1.0.0](https://toml.io/en/v1.0.0).
Below is each key and the expected values for that key.

#### Calculator Behavior

`calc.empty_push_behavior`: the behavior when an empty input line is evaluated. Must be one of the following values:
- `"None"`: do nothing
- `"Zero"`: push the number `0` onto the stack
- `"Last"`: push a copy of the bottom-most number onto the stack *(default)*

`calc.stack_size`: the size of the calculator's stack. Must be a positive integer. *(default `8`)*

#### Layout/Alignment

`tui.renderer.stack_alignment`: the text alignment of numbers on the stack. Must be one of the following values:
- `"Left"`: align numbers to the left *(default)*
- `"Right"`: align numbers to the right

`tui.renderer.memory_alignment`: the text alignment of keys and numbers in memory. Must be one of the following values:
- `"AllLeft"`: key and number aligned to the left
- `"SplitMiddle"`: key aligned to the left, number aligned to the right *(default)*
- `"AllRight"`: key and number aligned to the right

`tui.renderer.stack_location`: the position of the stack relative to the memory. Must be one of the following values:
- `"StackBottom"`: position the stack below memory
- `"StackTop"`: position the stack above memory
- `"StackRight"`: position the stack to the right of memory
- `"StackLeft"`: position the stack to the left of memory *(default)*

#### Colors

Each of the keys under `tui.renderer.colors` can have a color specified in a few ways.

The first way is with the color's name. These correspond 1 to 1 with the 8 dark and 8 light terminal colors.
Here are the possible values:

- `"Black"`
- `"Red"`
- `"Green"`
- `"Yellow"`
- `"Blue"`
- `"Magenta"`
- `"Cyan"`
- `"Gray"`
- `"DarkGray"`
- `"LightRed"`
- `"LightGreen"`
- `"LightYellow"`
- `"LightBlue"`
- `"LightMagenta"`
- `"LightCyan"`
- `"White"`

Note that color name parsing is more lenient than specified above, but it is not recommended to rely on that leniency -- it
can be broken at any time without warning.

The second way is with the ansi number code. A table can be found [here](https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit).
To specify a number with this method, write the value as `"NUMBER"`, where `NUMBER` is replaced with an integer from 0
to 255. Note that this is a string value, and not an integer value.

The third and final way is using RGB values. This is done as `"#RRGGBB"` where `RR`, `GG`, and `BB` are replaced with the values
of red, green, and blue in hexadecimal, respectively.

Here are the keys for colors:

`tui.renderer.colors.decimal_separator`: the color of the decimal separator character *(default `"LightCyan"`)*
`tui.renderer.colors.exponent_separator`: the color of the exponent separator character *(default `"LightCyan"`)*
`tui.renderer.colors.number`: the color of digits in a number *(default `"Green"`)*
`tui.renderer.colors.memory_key`: the color of the key in a memory entry *(default `"Red"`)*

#### Input

The TUI has two categories of input. The first is hotkeys/immediate commands. These are single button presses which
will perform some kind of action as soon as the press is registered. The second is string commands. These are bindings
between typed strings in the input buffer and an associated action when that buffer is evaluated.

An example of the first from the default config is `"S"` for swap. As soon as a capital S is received, the calculator will
swap the bottom two values of the stack.

An example of the second is `"sin"` for the sine function. When the input buffer reads `sin` and the buffer is evaluated,
the sine function will be performed on the bottom value of the stack.

This distinction is relevant for the following config options.

`tui.parser.imm_eval_mode`: how the input buffer is treated when an immediate command is performed. Must be one of the following:
- `"None"`: ignore the input buffer
- `"Numbers"`: if the input buffer is a valid number, push it onto the stack before executing the immediate command *(default)*
- `"Commands"`: if the input buffer is a valid string command, evaluate it before executing the immediate command
- `"All"`: combines `"Numbers"` and `"Commands"`

The following config keys map keyboard keys and strings to commands. For the actions below, `a` represents the second-bottom value of the stack
and `b` represents the bottom value of the stack. For each mapping, the values may be one of the following, which corresponds
to an action that can be performed:

- `"Quit"`: exit the program
- `"DelChar"`: delete the last character in the buffer
- `"EvalBuf"`: evaluate the current contents of the buffer, pushing in the case of a number or executing a string command otherwise
- `"Add"`: pop two values, compute `a + b`, and push the result
- `"Sub"`: pop two values, compute `a - b`, and push the result
- `"Mul"`: pop two values, compute `a * b`, and push the result
- `"Div"`: pop two values, compute `a / b`, and push the result
- `"Swp"`: swap the bottom two values on the stack
- `"Pow"`: pop two values, compute `a` to the power of `b`, and push the result
- `"Root"`: pop two values, compute `b`th root of `a`, and push the result
- `"Exp"`: pop two values, compute `a * 10^b` (`^` representing exponentiation), and push the result
- `"IntDiv"`: pop two values, compute the euclidean division of `a` by `b`
- `"Mod"`: pop two values, compute `a` modulo `b`
- `"Neg"`: swap the sign of the bottom value on the stack
- `"Sqrt"`: pops the bottom value, computes the square root, and pushes the result
- `"Sqr"`: pops the bottom value, squares it, and pushes the result
- `"Sin"`: pops the bottom value, computes the sine of the value, and pushes the result
- `"Cos"`: pops the bottom value, computes the cosine of the value, and pushes the result
- `"Tan"`: pops the bottom value, computes the tangent of the value, and pushes the result
- `"Asin"`: pops the bottom value, computes the arcsine of the value, and pushes the result
- `"Acos"`: pops the bottom value, computes the arccosine of the value, and pushes the result
- `"Atan"`: pops the bottom value, computes the arctangent of the value, and pushes the result
- `"Rad"`: pops the bottom value, converts it to radians from degrees, and pushes the result
- `"Deg"`: pops the bottom value, converts it to degrees from radians, and pushes the result
- `"Pop"`: pops the bottom value
- `"CalcStore"`: stores the bottom value into memory with the key given by the contents of the input buffer
- `"CalcDelete"`: removes the memory value with a key equal to the contents of the input buffer
- `"CalcRecall"`: pushes a value onto the stack from memory, using the contents of the input buffer as the key

`tui.parser.immediate_cmds`: a map between keys on the keyboard and commands. These are executed as soon as the keypress
is detected, and so anything bound here will not be able to be typed into the input buffer. This config key is a table,
where the values are one of the commands listed above and the keys may be one of the following:
- A single character representing a character that would be typed (like `"A"`, `"["`, or `"Ä"` if your keyboard supports typing such a character)
- A function key, written as `"fNUMBER"` where `NUMBER` is replaced with a non-negative integer (`"f5"` as an example for function key 5)
- The name of a key, from the following list:
    - `"left"`: left arrow
    - `"right"`: right arrow
    - `"up"`: up arrow
    - `"down"`: down arrow
    - `"home"`: home key
    - `"end"`: end key
    - `"pageup"`: page up key
    - `"pagedown"`: page down key
    - `"tab"`: tab key
    - `"backtab"`: tab key with shift held down
    - `"insert"`: insert key
    - `"esc"`: escape key
    - `"enter"`: the enter/return key
    - `"backspace"`: the backspace key
    - `"delete"`: the delete key

`tui.parser.string_cmds`: a map between strings (character sequences) and commands. These are executed when the `"EvalBuf"` command is executed and
the contents of the buffer do not parse as a number, and the contents match one of the entries under this key. The values of entries in this map
must be one of the commands listed above.

## Command-line Flags

The following command line flags are supported:

- `-c`, `--config-path`: takes a path to a file and uses that as the config file
- `-g`, `--generate-config`: generate a fresh config file at the location without prompting for confirmation,
overwriting an existing file if one is there. Requires `--config-path` to be supplied
- `-h`, `--help`: print help
- `-V`, `--version`: print version
