// Dark theme configuration inspired by JPEXS FFDec
export const theme = {
  // Background colors
  bg: {
    primary: '#1e1e1e',      // Main background
    secondary: '#252526',     // Sidebar, panels
    tertiary: '#2d2d30',      // Hover states
    elevated: '#3e3e42',      // Elevated elements
    input: '#3c3c3c',         // Input fields
  },

  // Text colors
  text: {
    primary: '#ffffff',       // Main text
    secondary: '#cccccc',     // Secondary text
    muted: '#858585',         // Muted text
    accent: '#4fc3f7',        // Links, accents
    success: '#4caf50',       // Success messages
    warning: '#ff9800',       // Warnings
    error: '#f44336',         // Errors
  },

  // Border colors
  border: {
    default: '#3e3e42',
    hover: '#4e4e52',
    focus: '#007acc',
  },

  // Syntax highlighting (for code)
  syntax: {
    keyword: '#569cd6',       // Keywords (class, function, var)
    string: '#ce9178',        // Strings
    number: '#b5cea8',        // Numbers
    comment: '#6a9955',       // Comments
    function: '#dcdcaa',      // Function names
    variable: '#9cdcfe',      // Variables
    type: '#4ec9b0',          // Types/classes
    operator: '#d4d4d4',      // Operators
  },

  // UI elements
  ui: {
    scrollbar: '#424242',
    scrollbarHover: '#4e4e4e',
    button: {
      primary: '#0e639c',
      primaryHover: '#1177bb',
      secondary: '#3e3e42',
      secondaryHover: '#4e4e52',
      success: '#0d7c24',
      successHover: '#0e8a28',
      warning: '#e67700',
      warningHover: '#f08000',
    },
    selection: {
      bg: '#264f78',
      border: '#007acc',
    },
    tree: {
      hover: '#2a2d2e',
      selected: '#37373d',
      icon: '#c5c5c5',
    },
  },

  // Resource type colors
  resourceColors: {
    image: '#f06292',
    sound: '#ba68c8',
    sprite: '#4fc3f7',
    script: '#aed581',
    text: '#ffb74d',
    font: '#a1887f',
    shape: '#4db6ac',
    binary: '#90a4ae',
  },
};

export type Theme = typeof theme;
