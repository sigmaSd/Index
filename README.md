# Index
Get any row,column combination from a terminal output

# Usage

printf $table | idx $row_format;$col_format

printf $table | idx 1,3;2

printf $table | idx ~3;2~

printf $table | idx ~3;_
