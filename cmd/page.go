package cmd

import (
	"bytes"
	"fmt"
	"os"
	"runtime/debug"
	"strings"
	"time"

	"github.com/PuerkitoBio/goquery"
	"github.com/gocolly/colly/v2"
)

type magnet struct {
	link    string
	size    int
	caption bool
}

type meta struct {
	gid  string
	uc   string
	img  string
	lang string
}

func getdetail(ds detail) {
	defer func() {
		curindex++
	}()
	filepath := output + "/" + ds.number
	if exist(filepath) {
		return
	}
	defer func() {
		if err := recover(); err != nil {
			fmt.Println(err)
			debug.PrintStack()
		}
	}()

	c, err := getrequest()
	if err != nil {
		return
	}
	link, number := ds.link, ds.number

	//var magnets []magnet
	c.SetRequestTimeout(time.Duration(timeout * 1e6))

	c.OnHTML("body", func(pe *colly.HTMLElement) {

		err := os.MkdirAll(filepath, 0777)
		if err != nil {
			fmt.Println(err)
		}
		pe.ForEach("script:nth-of-type(3)", func(i int, e *colly.HTMLElement) {
			if strings.Contains(e.Text, "gid") {
				magnets := getmagnetlist(parsescript(e.Text), link)
				//获取磁链
				ms := getmagnet(magnets)
				msf, err := os.Create(filepath + "/magnet.txt")
				if err != nil {
					fmt.Println(number, "磁力链接获取失败", err)
					return
				}
				defer msf.Close()
				_, _ = msf.WriteString(ms)
				fmt.Println(number, "磁力链接获取成功")
			}
		})
		if !nopic {
			pe.ForEach(".sample-box", func(i int, element *colly.HTMLElement) {
				imgurl := element.Attr("href")
				saveimg(imgurl, filepath)
			})
		}
	})
	err = c.Visit(link)
	if err != nil {
		fmt.Println(err)
	}
}

func exist(path string) bool {
	_, err := os.Stat(path)
	if err != nil {
		return os.IsExist(err)
	}
	return true
}

func getmagnetlist(m meta, link string) (magnets []magnet) {
	uri := baseurl + magneturl + "?" + m.gid + "&lang=" + m.lang + "&" + m.img + "&" + m.uc + "&floor=" + fmt.Sprint(randg.Intn(1e3))
	c, err := getrequest()
	if err != nil {
		return
	}
	c.OnRequest(func(request *colly.Request) {
		request.Headers.Set("Referer", link)
	})
	c.OnResponse(func(response *colly.Response) {
		bodys := "<html><body>" + string(response.Body) + "</body></html>"
		bodys = strings.Replace(bodys, "<tr", "<div", -1)
		bodys = strings.Replace(bodys, "tr>", "div>", -1)
		bodys = strings.Replace(bodys, "<td", "<p", -1)
		bodys = strings.Replace(bodys, "td>", "p>", -1)
		bf := bytes.NewReader([]byte(bodys))
		doc, err := goquery.NewDocumentFromReader(bf)
		if err != nil {
			fmt.Println(err)
		}
		doc.Find("div").Each(func(i int, s *goquery.Selection) {
			var mone magnet
			s.Find("p").Each(func(i2 int, s2 *goquery.Selection) {
				switch i2 {
				case 0:
					mone.link, _ = s2.Find("a[href]").Attr("href")
					mone.caption = s2.Find(".btn-warning").Text() == "字幕"
				case 1:
					mone.size = int(size(s2.Find("a[href]").Text()))
				}
			})
			magnets = append(magnets, mone)
		})
	})
	err = c.Visit(uri)
	if err != nil {
		fmt.Println(err)
	}
	return
}
