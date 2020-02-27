package cmd

import (
	"fmt"
	"math/rand"
	"os"
	"regexp"
	"time"

	homedir "github.com/mitchellh/go-homedir"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

var (
	parallel     int
	timeout      int
	limit        int
	output       string
	search       string
	base         string
	proxy        string
	nomag        bool
	allmag       bool
	nopic        bool
	caption      bool
	curpage      int   = 1
	curindex     int   = 0
	comindex     int64 = 0
	baseurl            = "https://www.javbus.com/"
	searchu            = "search/"
	magneturl          = "ajax/uncledatoolsbyajax.php"
	gidregexp    *regexp.Regexp
	ucregexp     *regexp.Regexp
	imgregexp    *regexp.Regexp
	numberregexp *regexp.Regexp
	randg        *rand.Rand
)

func init() {
	cobra.OnInitialize(initConfig)

	//
	var err error
	gidregexp, err = regexp.Compile(`var gid.*;`)
	if err != nil {
		fmt.Println(err)
	}
	ucregexp, err = regexp.Compile(`var uc.*;`)
	if err != nil {
		fmt.Println(err)
	}
	imgregexp, err = regexp.Compile(`var img.*;`)
	if err != nil {
		fmt.Println(err)
	}
	numberregexp, err = regexp.Compile(`[0-9]*\.[0-9]*`)
	if err != nil {
		fmt.Println(err)
	}
	//
	randg = rand.New(rand.NewSource(time.Now().UnixNano()))
	// Here you will define your flags and configuration settings.
	// Cobra supports persistent flags, which, if defined here,
	// will be global for your application.

	rootCmd.PersistentFlags().IntVarP(&parallel, "parallel", "p", 2, "设置每秒抓取请求书")
	rootCmd.PersistentFlags().IntVarP(&timeout, "timeout", "t", 30000, "自定义连接超时时间(毫秒)")
	rootCmd.PersistentFlags().IntVarP(&limit, "limit", "l", 0, "设置抓取影片的数量上限，0为抓取全部影片")
	rootCmd.PersistentFlags().StringVarP(&output, "output", "o", "magnets", "设置磁链和封面抓取结果的保存位置，默认为当前用户的主目录下的 magnets 文件夹")
	rootCmd.PersistentFlags().StringVarP(&search, "search", "s", "", "搜索关键词，可只抓取搜索结果的磁链或封面")
	rootCmd.PersistentFlags().StringVarP(&base, "base", "b", baseurl, "自定义抓取的起始页")
	rootCmd.PersistentFlags().StringVarP(&proxy, "proxy", "x", "", "使用代理服务器, 例：-x http://127.0.0.1:8087")
	rootCmd.PersistentFlags().BoolVarP(&nomag, "nomag", "n", false, "是否抓取尚无磁链的影片")
	rootCmd.PersistentFlags().BoolVarP(&allmag, "allmag", "a", false, "是否抓取尚无磁链的影片")
	rootCmd.PersistentFlags().BoolVarP(&nopic, "nopic", "i", false, "禁用图片抓取")
	rootCmd.PersistentFlags().BoolVarP(&caption, "caption", "c", false, "是否优先抓取有字幕的")
	// Cobra also supports local flags, which will only run
	// when this action is called directly.
}

// initConfig reads in config file and ENV variables if set.
func initConfig() {
	if cfgFile != "" {
		// Use config file from the flag.
		viper.SetConfigFile(cfgFile)
	} else {
		// Find home directory.
		home, err := homedir.Dir()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}

		// Search config in home directory with name ".jav" (without extension).
		viper.AddConfigPath(home)
		viper.SetConfigName(".jav")
	}

	viper.AutomaticEnv() // read in environment variables that match

	// If a config file is found, read it in.
	if err := viper.ReadInConfig(); err == nil {
		fmt.Println("Using config file:", viper.ConfigFileUsed())
	}
}

// Execute adds all child commands to the root command and sets flags appropriately.
// This is called by main.main(). It only needs to happen once to the rootCmd.
func Execute() {
	if err := rootCmd.Execute(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}
