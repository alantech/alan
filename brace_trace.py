import sys

with open('alan_compiler/src/lntors/function.rs', 'r') as f:
    content = f.read()

start_line = 855
end_line = 2661
lines = content.splitlines()
function_content = "\n".join(lines[start_line-1:end_line])

depth = 0
i = 0
while i < len(function_content):
    char = function_content[i]
    if char == '"':
        i += 1
        while i < len(function_content) and function_content[i] != '"':
            if function_content[i] == '\\':
                i += 1
            i += 1
    elif char == '{':
        depth += 1
    elif char == '}':
        depth -= 1
        if depth <= 0:
            line_num = function_content[:i].count('\n') + start_line
            print(f"Line {line_num}: depth {depth}")
    i += 1
