package formatter

import (
	"encoding/csv"
	"encoding/json"
	"fmt"
	"strings"

	"github.com/qmd/qmd-go/pkg/store"
)

// Format type
type Format string

const (
	FormatCLI     Format = "cli"
	FormatJSON    Format = "json"
	FormatMarkdown Format = "markdown"
	FormatCSV     Format = "csv"
	FormatFiles   Format = "files"
	FormatXML     Format = "xml"
)

// Formatter formats search results
type Formatter struct {
	format Format
	limit  int
}

// New creates a new Formatter
func New(format string, limit int) *Formatter {
	if limit == 0 {
		limit = 20
	}

	f := Format(format)
	if f == "" {
		f = FormatCLI
	}

	return &Formatter{
		format: f,
		limit:  limit,
	}
}

// FormatSearchResults formats search results
func (f *Formatter) FormatSearchResults(results []store.SearchResult) string {
	if len(results) > f.limit {
		results = results[:f.limit]
	}

	switch f.format {
	case FormatJSON:
		return f.formatJSON(results)
	case FormatMarkdown:
		return f.formatMarkdown(results)
	case FormatCSV:
		return f.formatCSV(results)
	case FormatFiles:
		return f.formatFiles(results)
	case FormatXML:
		return f.formatXML(results)
	default:
		return f.formatCLI(results)
	}
}

func (f *Formatter) formatCLI(results []store.SearchResult) string {
	if len(results) == 0 {
		return "No results found.\n"
	}

	var sb strings.Builder
	sb.WriteString(fmt.Sprintf("%-10s %-8s %-6s %s\n", "Score", "Lines", "DocID", "Path"))
	sb.WriteString(fmt.Sprintf("%-10s %-8s %-6s %s\n", "------", "-----", "-----", "----"))

	for _, r := range results {
		sb.WriteString(fmt.Sprintf("%-10.4f %-8d %-6d %s\n", r.Score, r.Lines, r.DocID, r.Path))
	}

	sb.WriteString(fmt.Sprintf("\nTotal: %d results\n", len(results)))
	return sb.String()
}

func (f *Formatter) formatJSON(results []store.SearchResult) string {
	type Output struct {
		Query  string            `json:"query"`
		Total  int               `json:"total"`
		Results []store.SearchResult `json:"results"`
	}

	output := Output{
		Query:   "",
		Total:   len(results),
		Results: results,
	}

	data, _ := json.MarshalIndent(output, "", "  ")
	return string(data) + "\n"
}

func (f *Formatter) formatMarkdown(results []store.SearchResult) string {
	var sb strings.Builder
	sb.WriteString("# Search Results\n\n")
	sb.WriteString(fmt.Sprintf("Total: %d results\n\n", len(results)))

	for i, r := range results {
		sb.WriteString(fmt.Sprintf("## %d. %s\n\n", i+1, r.Path))
		sb.WriteString(fmt.Sprintf("- **Score**: %.4f\n", r.Score))
		sb.WriteString(fmt.Sprintf("- **Lines**: %d\n", r.Lines))
		sb.WriteString(fmt.Sprintf("- **Collection**: %s\n", r.Collection))
		sb.WriteString(fmt.Sprintf("- **DocID**: %s\n\n", r.DocID))
	}

	return sb.String()
}

func (f *Formatter) formatCSV(results []store.SearchResult) string {
	var sb strings.Builder
	writer := csv.NewWriter(&sb)
	defer writer.Flush()

	// Header
	writer.Write([]string{"score", "lines", "docid", "path", "collection", "title", "hash"})

	// Data
	for _, r := range results {
		writer.Write([]string{
			fmt.Sprintf("%.4f", r.Score),
			fmt.Sprintf("%d", r.Lines),
			r.DocID,
			r.Path,
			r.Collection,
			r.Title,
			r.Hash,
		})
	}

	return sb.String()
}

func (f *Formatter) formatFiles(results []store.SearchResult) string {
	var sb strings.Builder
	for _, r := range results {
		sb.WriteString(r.Path)
		sb.WriteString("\n")
	}
	return sb.String()
}

func (f *Formatter) formatXML(results []store.SearchResult) string {
	var sb strings.Builder
	sb.WriteString("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")
	sb.WriteString("<results>\n")
	sb.WriteString(fmt.Sprintf("  <total>%d</total>\n", len(results)))

	for _, r := range results {
		sb.WriteString("  <result>\n")
		sb.WriteString(fmt.Sprintf("    <score>%.4f</score>\n", r.Score))
		sb.WriteString(fmt.Sprintf("    <lines>%d</lines>\n", r.Lines))
		sb.WriteString(fmt.Sprintf("    <docid>%s</docid>\n", r.DocID))
		sb.WriteString(fmt.Sprintf("    <path>%s</path>\n", escapeXML(r.Path)))
		sb.WriteString(fmt.Sprintf("    <collection>%s</collection>\n", r.Collection))
		sb.WriteString(fmt.Sprintf("    <title>%s</title>\n", escapeXML(r.Title)))
		sb.WriteString(fmt.Sprintf("    <hash>%s</hash>\n", r.Hash))
		sb.WriteString("  </result>\n")
	}

	sb.WriteString("</results>\n")
	return sb.String()
}

func escapeXML(s string) string {
	s = strings.ReplaceAll(s, "&", "&amp;")
	s = strings.ReplaceAll(s, "<", "&lt;")
	s = strings.ReplaceAll(s, ">", "&gt;")
	s = strings.ReplaceAll(s, "\"", "&quot;")
	s = strings.ReplaceAll(s, "'", "&apos;")
	return s
}
