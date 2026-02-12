package cli

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

var mcpCmd = &cobra.Command{
	Use:   "mcp",
	Short: "Run as MCP server",
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("MCP server not implemented in Go version")
		os.Exit(1)
	},
}
