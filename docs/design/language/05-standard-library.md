# 05 Standard Library

05 - SF 标准库

1. 概述

SF 语言内置标准库，提供常用函数和常量，无需安装扩展即可使用。标准库在编译时自动链接，不增加运行时开销。

2. 数学函数 (math)

· abs(x): 绝对值
· sqrt(x): 平方根
· sin(x), cos(x), tan(x): 三角函数（弧度制）
· asin(x), acos(x), atan(x): 反三角函数
· ln(x), log10(x), exp(x): 对数和指数
· round(x), floor(x), ceil(x): 取整
· random(min, max): 随机数 [min, max]
· clamp(x, min, max): 钳制值到指定范围
· lerp(a, b, t): 线性插值
· PI, E: 数学常量

3. 字符串函数 (string)

· length(s): 字符串长度
· substring(s, start, end): 子串
· indexOf(s, target): 查找子串位置，未找到返回 -1
· replace(s, old, new): 替换
· toUpper(s), toLower(s): 大小写转换
· trim(s): 去除首尾空格
· split(s, delimiter): 分割为列表
· join(list, delimiter): 列表合并为字符串

4. 时间函数 (time)

· now(): 返回当前时间戳 (毫秒，自 1970-01-01 UTC)
· timer(): 返回项目运行时长 (秒，从绿旗点击或 resetTimer() 调用开始计时)
· resetTimer(): 重置项目计时器
· daysSince2000(): 返回自 2000-01-01 起的天数
