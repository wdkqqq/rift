<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { onboardingOpen } from "../stores/app";
    import { cubicInOut, cubicOut } from "svelte/easing";
    import { fade } from "svelte/transition";
    import { onMount, tick } from "svelte";

    type OnboardingOption = {
        label: string;
        hint: string;
    };

    type OnboardingStep = {
        title: string;
        description: string;
        options: OnboardingOption[];
    };

    const steps: OnboardingStep[] = [
        {
            title: "Where is your music folder?",
            description:
                "Rift can use your default Music folder immediately. You can change it any time later in settings.",
            options: [
                {
                    label: "Use default Music folder",
                    hint: "~/Music",
                },
                {
                    label: "I'll choose it later",
                    hint: "Keep current defaults for now",
                },
            ],
        },
        {
            title: "Enable automatic updates?",
            description:
                "Keep Rift up to date automatically so fixes and improvements arrive without manual downloads.",
            options: [
                {
                    label: "Yes, keep Rift updated",
                    hint: "Install updates automatically",
                },
                {
                    label: "No, I'll update manually",
                    hint: "Stay on current version until you decide",
                },
            ],
        },
        {
            title: "Allow internet requests?",
            description:
                "Needed for artist images and extra metadata lookups. If you prefer strict offline mode, you can keep this off.",
            options: [
                {
                    label: "Yes, allow requests",
                    hint: "Enable richer metadata and artist artwork",
                },
                {
                    label: "No, keep it offline",
                    hint: "Stay local-only with no external requests",
                },
            ],
        },
    ];

    let currentStep = $state(0);
    let selectedOption = $state<number | null>(null);
    let selectedOptions = $state<(number | null)[]>(
        Array.from({ length: steps.length }, () => null),
    );
    let progress = $derived((currentStep + 1) / steps.length);
    let isAnimating = false;
    let queuedStep: number | null = null;
    let isMac = false;
    let stepContentStyle = $state(
        "opacity: 1; transform: translateY(0); transition: opacity 240ms ease, transform 240ms ease;",
    );
    let stepTimer: ReturnType<typeof setTimeout> | null = null;

    function wait(ms: number) {
        return new Promise<void>((resolve) => {
            stepTimer = setTimeout(resolve, ms);
        });
    }

    function isMacOS() {
        if (document.body.classList.contains("macos-traffic-lights")) {
            return true;
        }
        const userAgentDataPlatform = navigator.userAgentData?.platform ?? "";
        const platform = navigator.platform ?? "";
        const userAgent = navigator.userAgent ?? "";
        return /mac/i.test(userAgentDataPlatform + platform + userAgent);
    }

    function closeOnboarding() {
        if (stepTimer) clearTimeout(stepTimer);
        onboardingOpen.set(false);
        currentStep = 0;
        selectedOption = null;
        selectedOptions = Array.from({ length: steps.length }, () => null);
        isAnimating = false;
        queuedStep = null;
        stepContentStyle =
            "opacity: 1; transform: translateY(0); transition: opacity 240ms ease, transform 240ms ease;";
    }

    function chooseOption(optionIndex: number) {
        if (isAnimating) return;
        selectedOption = optionIndex;
        selectedOptions[currentStep] = optionIndex;
    }

    async function animateStepSwitch(nextStepIndex: number) {
        if (isAnimating) {
            queuedStep = nextStepIndex;
            return;
        }

        if (nextStepIndex === currentStep) return;

        isAnimating = true;
        stepContentStyle =
            "opacity: 0; transform: translateY(-12px); transition: opacity 150ms ease, transform 150ms ease;";
        await wait(120);

        currentStep = nextStepIndex;
        selectedOption = selectedOptions[nextStepIndex];
        await tick();

        stepContentStyle =
            "opacity: 0; transform: translateY(10px); transition: none;";
        await tick();

        requestAnimationFrame(() => {
            stepContentStyle =
                "opacity: 1; transform: translateY(0); transition: opacity 240ms ease, transform 240ms ease;";
        });
        await wait(250);

        isAnimating = false;
        if (queuedStep !== null && queuedStep !== currentStep) {
            const pending = queuedStep;
            queuedStep = null;
            await animateStepSwitch(pending);
        } else {
            queuedStep = null;
        }
    }

    async function nextStep() {
        if (isAnimating || selectedOption === null) return;
        if (currentStep < steps.length - 1) {
            await animateStepSwitch(currentStep + 1);
            return;
        }
        try {
            await invoke("set_onboarding_played", { played: true });
        } catch (error) {
            console.error("Failed to persist onboarding completion", error);
        }
        closeOnboarding();
    }

    async function prevStep() {
        if (isAnimating || currentStep === 0) return;
        await animateStepSwitch(currentStep - 1);
    }

    async function handleTopEdgeMouseDown(event: MouseEvent) {
        if (!isMac) return;
        if (event.button !== 0) return;
        try {
            await getCurrentWindow().startDragging();
        } catch (error) {
            console.error("Failed to start window dragging", error);
        }
    }

    onMount(() => {
        isMac = isMacOS();
    });
</script>

<div
    class="fixed inset-0 z-[70] overflow-hidden bg-[#090909]"
    in:fade={{ duration: 280, easing: cubicOut }}
    out:fade={{ duration: 320, easing: cubicInOut }}
>
    <div
        class="absolute top-0 left-0 right-0 h-12 z-[90] pointer-events-auto select-none"
        onmousedown={handleTopEdgeMouseDown}
    ></div>
    <div class="absolute inset-0 bg-black/30"></div>

    <div class="relative z-10 h-full w-full">
        <div
            class="mx-auto flex h-full max-w-2xl flex-col px-5 py-7 md:px-8 md:py-8"
        >
            <div class="h-1.5 w-full rounded-full bg-[#2a2a2a] overflow-hidden">
                <div
                    class="h-full bg-white transition-all duration-500 ease-out"
                    style:width={`${progress * 100}%`}
                ></div>
            </div>

            <div class="flex-1 flex items-center justify-center">
                <section
                    class="w-full max-w-xl mx-auto"
                    style={stepContentStyle}
                >
                    <h2
                        class="mb-3 text-white text-2xl md:text-3xl font-semibold leading-[1.12] text-center"
                    >
                        {steps[currentStep].title}
                    </h2>
                    <p
                        class="mb-6 max-w-lg mx-auto text-secondary text-xs md:text-sm leading-relaxed text-center"
                    >
                        {steps[currentStep].description}
                    </p>

                    <div class="grid gap-3">
                        {#each steps[currentStep].options as option, index}
                            <button
                                type="button"
                                class="group active:scale-95 rounded-lg bg-[#111111]/85 px-4 py-3 text-left hover:bg-hover [transition:all_0.2s_ease] {selectedOption ===
                                index
                                    ? 'bg-hover'
                                    : ''}"
                                onclick={() => chooseOption(index)}
                            >
                                <p class="text-white text-sm md:text-base mb-1">
                                    {option.label}
                                </p>
                                <p class="text-secondary text-xs">
                                    {option.hint}
                                </p>
                            </button>
                        {/each}
                    </div>
                    <div class="mt-5 flex items-center justify-between gap-3">
                        <button
                            type="button"
                            disabled={currentStep === 0}
                            class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 text-xs [transition:all_0.2s_ease] {currentStep ===
                            0
                                ? 'text-[#555555] pointer-events-none'
                                : 'text-secondary hover:text-white hover:bg-[#1a1a1a] active:scale-95'}"
                            onclick={() => void prevStep()}
                        >
                            <span aria-hidden="true">←</span>
                            <span>Back</span>
                        </button>
                        <button
                            type="button"
                            class="inline-flex items-center rounded-md px-3 py-1.5 text-xs text-white bg-[#1f1f1f] hover:bg-hover active:scale-95 [transition:all_0.2s_ease] {selectedOption ===
                            null
                                ? 'opacity-50 pointer-events-none'
                                : ''}"
                            onclick={() => void nextStep()}
                        >
                            {currentStep === steps.length - 1
                                ? "Finish"
                                : "Next"}
                        </button>
                    </div>
                </section>
            </div>
        </div>
    </div>
</div>
