# Index
Get any row,column combination from a terminal output

# Usage

`printf $table | idx $row_format;$col_format`

`$row/col_format` can be:
- a serie of numbers sperated by a comma ex: 1,2
- a range sperated by `~`, ex: 1~ ex: 1~2 ex: ~3
- catch all with `_`

# Example
printf $table | idx 1,3;2

printf $table | idx \~3;2~

printf $table | idx ~3;_
