<script lang="ts">
  // Daily Stoic — quotes from Marcus Aurelius, Epictetus, Seneca.
  // Tap to save favourites, rotate through collection.

  let saved = $state<Set<string>>(new Set());
  let index = $state(0);

  const quotes: { text: string; author: string; source: string }[] = [
    { text: "You have power over your mind — not outside events. Realize this, and you will find strength.", author: "Marcus Aurelius", source: "Meditations" },
    { text: "The impediment to action advances action. What stands in the way becomes the way.", author: "Marcus Aurelius", source: "Meditations" },
    { text: "Waste no more time arguing about what a good man should be. Be one.", author: "Marcus Aurelius", source: "Meditations" },
    { text: "When you arise in the morning, think of what a precious privilege it is to be alive — to breathe, to think, to enjoy, to love.", author: "Marcus Aurelius", source: "Meditations" },
    { text: "The happiness of your life depends upon the quality of your thoughts.", author: "Marcus Aurelius", source: "Meditations" },
    { text: "It is not death that a man should fear, but he should fear never beginning to live.", author: "Marcus Aurelius", source: "Meditations" },
    { text: "We suffer more often in imagination than in reality.", author: "Seneca", source: "Letters to Lucilius" },
    { text: "Luck is what happens when preparation meets opportunity.", author: "Seneca", source: "Letters to Lucilius" },
    { text: "While we wait for life, life passes.", author: "Seneca", source: "On the Shortness of Life" },
    { text: "Difficulties strengthen the mind, as labor does the body.", author: "Seneca", source: "On Providence" },
    { text: "Every day is a new life to a wise man.", author: "Seneca", source: "Letters to Lucilius" },
    { text: "It's not what happens to you, but how you react to it that matters.", author: "Epictetus", source: "Enchiridion" },
    { text: "Wealth consists not in having great possessions, but in having few wants.", author: "Epictetus", source: "Enchiridion" },
    { text: "He is a wise man who does not grieve for the things which he has not, but rejoices for those which he has.", author: "Epictetus", source: "Discourses" },
    { text: "No man is free who is not master of himself.", author: "Epictetus", source: "Enchiridion" },
    { text: "Don't explain your philosophy. Embody it.", author: "Epictetus", source: "Discourses" },
    { text: "The soul becomes dyed with the colour of its thoughts.", author: "Marcus Aurelius", source: "Meditations" },
    { text: "Accept the things to which fate binds you, and love the people with whom fate brings you together.", author: "Marcus Aurelius", source: "Meditations" },
    { text: "The best revenge is to be unlike him who performed the injury.", author: "Marcus Aurelius", source: "Meditations" },
    { text: "Confine yourself to the present.", author: "Marcus Aurelius", source: "Meditations" },
  ];

  const current = $derived(quotes[index % quotes.length]);
  const isSaved = $derived(saved.has(current.text));

  function next() { index++; }
  function prev() { index = Math.max(0, index - 1); }
  function toggleSave() {
    if (isSaved) { saved.delete(current.text); } else { saved.add(current.text); }
    saved = new Set(saved); // trigger reactivity
  }
</script>

<section class="stoic flex h-full w-full flex-col justify-center px-8 gap-3">
  <div class="header flex items-center justify-between">
    <span class="label-track text-[#3a8b9e]">Stoic Daily</span>
    <button class="save-btn" class:saved={isSaved} onclick={toggleSave} title="Save favourite">
      {isSaved ? "★" : "☆"}
    </button>
  </div>

  <blockquote class="quote font-display text-base text-[#e8eef2] leading-relaxed italic">
    "{current.text}"
  </blockquote>

  <div class="attribution flex items-center justify-between">
    <span class="font-display text-sm text-[#6b7785]">— {current.author}</span>
    <span class="font-data text-[10px] text-[#6b7785]/60">{current.source}</span>
  </div>

  <div class="nav flex items-center gap-4 mt-1">
    <button class="nav-btn" onclick={prev}>‹</button>
    <span class="font-data text-[10px] text-[#6b7785]">{(index % quotes.length) + 1} / {quotes.length}</span>
    <button class="nav-btn" onclick={next}>›</button>
  </div>
</section>

<style>
  .label-track { letter-spacing: 0.18em; text-transform: uppercase; font-size: 0.625rem; font-weight: 500; }
  .save-btn { background: none; border: none; font-size: 16px; cursor: pointer; color: #6b7785; }
  .save-btn.saved { color: #00d9ff; }
  .nav-btn { background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); color: #6b7785; padding: 2px 10px; border-radius: 3px; font-size: 14px; cursor: pointer; }
  .nav-btn:hover { color: #e8eef2; background: rgba(255,255,255,0.08); }
</style>