/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      // 🚀 修复 h-4.5 报错：添加自定义间距
      spacing: {
        '4.5': '1.125rem', // 18px
      }
    },
  },
  plugins: [],
}