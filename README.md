# jav

Crawl javbus Magnet

## Install

```bash
go get github.com/hyperq/jav
```

## Usage

```bash
➜ jav -h
--------------------------------------------------------------------------------------------------------------
javbus spider written by go

Usage:
  jav [flags]

Flags:
  -a, --allmag          是否抓取尚无磁链的影片
  -b, --base string     自定义抓取的起始页 (default "https://www.javbus.com/")
  -c, --caption         是否优先抓取有字幕的
  -h, --help            help for jav
  -l, --limit int       设置抓取影片的数量上限，0为抓取全部影片
  -n, --nomag           是否抓取尚无磁链的影片
  -i, --nopic           禁用图片抓取
  -o, --output string   设置磁链和封面抓取结果的保存位置，默认为当前用户的主目录下的 magnets 文件夹 (default "magnets")
  -p, --parallel int    设置每秒抓取请求书 (default 2)
  -x, --proxy string    使用代理服务器, 例：-x http://127.0.0.1:8087
  -s, --search string   搜索关键词，可只抓取搜索结果的磁链或封面
  -t, --timeout int     自定义连接超时时间(毫秒) (default 30000)
```

因为 javbus 国内有限制,请使用科学上网工具

```bash
jav -x http://127.0.0.1:8087
```
