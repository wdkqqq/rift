<script lang="ts">
  import {
    Shuffle,
    SkipBack,
    Play,
    SkipForward,
    Repeat,
    Heart,
    Volume2,
  } from "lucide-svelte";
</script>

<div class="p-3 border-t border-[#212121]">
  <div class="flex items-center justify-between pl-2">
    <div class="flex items-center flex-1">
      <div
        class="w-11 h-11 bg-[#0F0F0F] rounded mr-3 shrink-0 flex items-center justify-center"
      >
        <svg
          class="h-5 w-5 text-[#A3A3A3]"
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"
          />
        </svg>
      </div>
      <div class="flex items-center">
        <div class="mr-3">
          <div class="text-white">Bad for Business</div>
          <div class="text-sm text-[#A0A0A0]">Sabrina Carpenter</div>
        </div>
        <button class="text-[#A0A0A0] hover:text-white transition-colors p-1">
          <Heart class="h-4 w-4" />
        </button>
      </div>
    </div>

    <div class="flex flex-col items-center space-y-2 w-1/3">
      <div class="flex items-center space-x-6">
        <button class="text-[#A0A0A0] transition-colors">
          <Shuffle class="h-5 w-5" />
        </button>

        <button class="text-[#A0A0A0] transition-colors">
          <SkipBack class="h-5 w-5" />
        </button>

        <button class="text-[#A0A0A0] p-2 transition-transform">
          <Play class="h-5 w-5" />
        </button>

        <button class="text-[#818181] transition-colors">
          <SkipForward class="h-5 w-5" />
        </button>

        <button class="text-[#818181] transition-colors">
          <Repeat class="h-5 w-5" />
        </button>
      </div>

      <div
        class="flex items-center space-x-2 text-xs text-[#A0A0A0] w-full max-w-md"
      >
        <span>0:00</span>
        <div class="flex-1 h-1 bg-[#404040] rounded-full">
          <div class="h-1 bg-white rounded-full w-1/3"></div>
        </div>
        <span>3:35</span>
      </div>
    </div>

    <div class="flex items-center justify-end space-x-3 w-1/3 pr-2">
      <button class="text-[#A0A0A0] hover:text-white transition-colors">
        <Volume2 class="h-5 w-5" />
      </button>

      <div class="w-24">
        <div class="h-1 bg-[#404040] rounded-full">
          <div class="h-1 bg-white rounded-full w-2/3"></div>
        </div>
      </div>
    </div>
  </div>
</div>
