package cmd

import (
	"bytes"
	"fmt"
	"strconv"
	"strings"
	"time"

	"github.com/PuerkitoBio/goquery"

	"github.com/gocolly/colly/v2"
)

type detail struct {
	link   string
	number string
}

func get(re int) {
	c, err := getrequest()
	if err != nil {
		return
	}

	c.OnResponse(func(r *colly.Response) {
		bf := bytes.NewReader([]byte(r.Body))
		doc, err := goquery.NewDocumentFromReader(bf)
		if err != nil {
			fmt.Println(err)
		}
		box := doc.Find(".movie-box")
		var ds []detail
		box.Each(func(i int, e *goquery.Selection) {
			link, _ := e.Attr("href")
			number := e.Find("date:nth-of-type(1)").Text()
			d := detail{
				link:   link,
				number: number,
			}
			ds = append(ds, d)
		})
		if len(ds) == 0 {
			if re < 4 {
				fmt.Printf("第%d页获取失败,重试第%d次", curpage, re)
				re++
				get(re)
			} else {
				end = true
				return
			}
		}
		getdetaillist(ds)
	})
	rurl := base
	if search != "" {
		rurl += searchu + search
	}
	if curpage != 1 {
		rurl += "/page/" + strconv.Itoa(curpage)
	}
	fmt.Printf("正在获取第%d页\n", curpage)
	err = c.Visit(rurl)
	if err != nil {
		fmt.Println(err)
		return
	}
}

func getdetaillist(ds []detail) {
	fmt.Println("本页番号为:", rangenumber(ds))
	length := len(ds)
	for k := range ds {
		if curindex >= limit && limit > 0 {
			return
		}
		if curindex/parallel == 0 {
			time.Sleep(time.Second)
		}
		curindex++
		go getdetail(ds[k])
		if k == length-1 {
			curpage++
			get(1)
		}
	}
}

func rangenumber(ds []detail) string {
	var ss []string
	for k := range ds {
		ss = append(ss, ds[k].number)
	}
	return strings.Join(ss, ",")
}
