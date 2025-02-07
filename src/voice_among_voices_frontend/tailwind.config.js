/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    fontFamily: {
      sans: ['Satoshi-Variable', 'system-ui', 'sans-serif'],
      italic: ['Satoshi-VariableItalic', 'system-ui', 'sans-serif'],
    },
    extend: {
      fontWeight: {
        light: '300',
        normal: '400',
        medium: '500',
        bold: '700',
        black: '900',
      },
      fontFamily: {
        'sans-light': ['Satoshi-Light', 'system-ui', 'sans-serif'],
        'sans-regular': ['Satoshi-Regular', 'system-ui', 'sans-serif'],
        'sans-medium': ['Satoshi-Medium', 'system-ui', 'sans-serif'],
        'sans-bold': ['Satoshi-Bold', 'system-ui', 'sans-serif'],
        'sans-black': ['Satoshi-Black', 'system-ui', 'sans-serif'],
      }
    },
  },
  plugins: [],
}

