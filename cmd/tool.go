package cmd

import (
	"bufio"
	"fmt"
	"io"
	"net/http"
	"os"
	"path"
	"strconv"
	"strings"
	"time"

	"github.com/gocolly/colly/v2"
)

func replace(s string) string {
	s = strings.Replace(s, "var", "", -1)
	s = strings.Replace(s, ";", "", -1)
	s = strings.Replace(s, " ", "", -1)
	s = strings.Replace(s, "'", "", -1)
	return s
}

func size(s string) float64 {
	number := numberregexp.FindString(s)
	numberi, _ := strconv.ParseFloat(number, 64)
	s = strings.Replace(s, number, "", -1)
	s = strings.Replace(s, " ", "", -1)
	s = strings.Replace(s, "    ", "", -1)
	var modulus float64
	if strings.Index(s, "G") > -1 {
		modulus = 1024
	}
	if strings.Index(s, "M") > -1 {
		modulus = 1
	}
	return numberi * modulus
}

func getmagnet(ms []magnet) (link string) {
	var links []string
	var maxsize int
	for k := range ms {
		if allmag {
			links = append(links, ms[k].link)
		} else {
			if ms[k].size > maxsize {
				maxsize = ms[k].size
				if len(links) == 0 {
					links = append(links, ms[k].link)
				} else {
					links[0] = ms[k].link
				}
			}
		}
	}
	if caption {
		maxsize = 0
		for k := range ms {
			if ms[k].caption && ms[k].size > maxsize {
				maxsize = ms[k].size
				if len(links) == 0 {
					links = append(links, ms[k].link)
				} else {
					links[0] = ms[k].link
				}
			}
		}
	}
	link = strings.Join(links, "\n")
	return
}

func saveimg(imgurl string, filepath string) {
	fileName := path.Base(imgurl)
	fmt.Println("获取图片:" + fileName)
	resp, err := http.Get(imgurl)
	if err != nil {
		fmt.Println(fileName, "图片获取失败:", err)
		return
	}
	defer resp.Body.Close()
	reader := bufio.NewReaderSize(resp.Body, 32*1024)
	file, err := os.Create(filepath + "/" + fileName)
	if err != nil {
		fmt.Println(fileName, "图片获取失败:", err)
		return
	}
	defer file.Close()
	_, err = io.Copy(file, reader)
	if err != nil {
		fmt.Println(fileName, "图片获取失败:", err)
		return
	}
}

func parsescript(s string) (m meta) {
	m.gid = replace(gidregexp.FindString(s))
	m.uc = replace(ucregexp.FindString(s))
	m.img = replace(imgregexp.FindString(s))
	m.lang = "zh"
	return
}

func getrequest() (c *colly.Collector, err error) {
	c = colly.NewCollector()
	if proxy != "" {
		err = c.SetProxy(proxy)
		if err != nil {
			fmt.Println(err)
			return
		}
	}
	c.SetRequestTimeout(time.Duration(timeout * 1e6))
	return
}
