package cli

import (
	"fmt"

	"github.com/spf13/cobra"
)

var mcpCmd = &cobra.Command{
	Use:   "mcp",
	Short: "Run as MCP server",
	Run: func(cmd *cobra.Command, args []string) {
		transport, _ := cmd.Flags().GetString("transport")
		port, _ := cmd.Flags().GetUint("port")

		// Check for dry-run mode
		dryRun, _ := cmd.Flags().GetBool("dry-run")
		if dryRun {
			fmt.Println("[DRY-RUN] Would execute mcp server with:")
			fmt.Printf("  transport: %s\n", transport)
			fmt.Printf("  port: %d\n", port)
			return
		}

		fmt.Printf("Starting MCP server (transport: %s, port: %d)\n", transport, port)
	},
}

var agentCmd = &cobra.Command{
	Use:   "agent",
	Short: "Run in agent mode",
	Run: func(cmd *cobra.Command, args []string) {
		interactive, _ := cmd.Flags().GetBool("interactive")
		mcp, _ := cmd.Flags().GetBool("mcp")
		transport, _ := cmd.Flags().GetString("transport")

		if interactive {
			fmt.Println("Agent mode - interactive")
			fmt.Println("Type 'exit' to quit")
		} else {
			fmt.Println("Agent mode ready")
		}

		if mcp {
			fmt.Printf("MCP server enabled (transport: %s)\n", transport)
		}
	},
}

func init() {
	mcpCmd.Flags().StringP("transport", "t", "stdio", "Transport: stdio, sse")
	mcpCmd.Flags().UintP("port", "p", 8080, "Port for SSE transport")

	agentCmd.Flags().BoolP("interactive", "i", false, "Interactive mode")
	agentCmd.Flags().Bool("mcp", false, "Also run MCP server")
	agentCmd.Flags().StringP("transport", "t", "stdio", "MCP transport")
}
