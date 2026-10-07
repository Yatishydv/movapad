import Image from "next/image";

const Logo = () => (
  <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" className="text-black dark:text-white">
    <rect x="2" y="4" width="20" height="16" rx="4"></rect>
    <path d="M2 15h20"></path>
  </svg>
);

export default function Home() {
  return (
    <main className="min-h-screen bg-white dark:bg-[#111111] text-neutral-900 dark:text-neutral-100 font-sans selection:bg-neutral-200 dark:selection:bg-neutral-800 flex flex-col">
      
      {/* Navigation */}
      <nav className="w-full px-8 py-10 flex justify-between items-center">
        <div className="flex items-center gap-3">
          <Logo />
          <span className="font-medium tracking-tight">MovaPad</span>
        </div>
        <div className="flex gap-6 text-sm text-neutral-500 hover:text-neutral-900 dark:hover:text-neutral-300 transition-colors">
          <a href="https://github.com/yatishydv/movapad" target="_blank" rel="noreferrer">GitHub</a>
        </div>
      </nav>

      {/* Hero Section */}
      <div className="flex-1 flex flex-col justify-center px-8 max-w-4xl pb-32">
        <h1 className="text-5xl md:text-7xl font-medium tracking-tight mb-8 leading-[1.05]">
          Your smartphone is the trackpad.
        </h1>
        
        <p className="text-xl md:text-2xl text-neutral-500 dark:text-neutral-400 mb-12 max-w-2xl font-normal leading-snug">
          MovaPad connects your phone to your computer over the local network to act as a wireless mouse. It's fast, free, and runs entirely on your device.
        </p>

        <div className="flex flex-col sm:flex-row gap-4 w-full sm:w-auto">
          <button className="px-7 py-3.5 rounded-lg bg-neutral-900 dark:bg-white text-white dark:text-black font-medium hover:bg-neutral-800 dark:hover:bg-neutral-200 transition-colors">
            Download for macOS
          </button>
          <button className="px-7 py-3.5 rounded-lg bg-neutral-100 dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100 font-medium hover:bg-neutral-200 dark:hover:bg-neutral-800 transition-colors">
            Get the Mobile App
          </button>
        </div>
      </div>
      
    </main>
  );
}
