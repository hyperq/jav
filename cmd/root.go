package cmd

import (
	"fmt"
	"time"

	"github.com/spf13/cobra"
)

var cfgFile string

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
	Use:   "jav",
	Short: "javbus spider written by go",
	Run: func(cmd *cobra.Command, args []string) {
		get(1)
		for {
			time.Sleep(time.Second)
			if end {
				fmt.Println("抓取完成")
				return
			}
		}
	},
}
