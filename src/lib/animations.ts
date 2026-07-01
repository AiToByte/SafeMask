import type { Variants, Transition } from "framer-motion";

// ── Shared spring transition ──
export const spring: Transition = {
  type: "spring",
  stiffness: 300,
  damping: 30,
};

export const springGentle: Transition = {
  type: "spring",
  stiffness: 200,
  damping: 25,
};

export const springSnappy: Transition = {
  type: "spring",
  stiffness: 500,
  damping: 35,
};

// ── Page / container variants ──
export const pageTransition: Variants = {
  initial: { opacity: 0, y: 12 },
  animate: {
    opacity: 1,
    y: 0,
    transition: { duration: 0.4, ease: [0.16, 1, 0.3, 1] },
  },
  exit: { opacity: 0, y: -8, transition: { duration: 0.25 } },
};

export const fadeIn: Variants = {
  initial: { opacity: 0 },
  animate: { opacity: 1, transition: { duration: 0.5 } },
  exit: { opacity: 0, transition: { duration: 0.2 } },
};

export const fadeInUp: Variants = {
  initial: { opacity: 0, y: 20 },
  animate: {
    opacity: 1,
    y: 0,
    transition: { duration: 0.5, ease: [0.16, 1, 0.3, 1] },
  },
};

export const scaleIn: Variants = {
  initial: { opacity: 0, scale: 0.95 },
  animate: {
    opacity: 1,
    scale: 1,
    transition: springGentle,
  },
  exit: { opacity: 0, scale: 0.95, transition: { duration: 0.2 } },
};

// ── Stagger children ──
export const staggerContainer: Variants = {
  initial: {},
  animate: {
    transition: { staggerChildren: 0.06, delayChildren: 0.05 },
  },
};

export const staggerItem: Variants = {
  initial: { opacity: 0, y: 16 },
  animate: {
    opacity: 1,
    y: 0,
    transition: springGentle,
  },
};

// ── Slide directions ──
export const slideInLeft: Variants = {
  initial: { opacity: 0, x: -20 },
  animate: { opacity: 1, x: 0, transition: springGentle },
};

export const slideInRight: Variants = {
  initial: { opacity: 0, x: 20 },
  animate: { opacity: 1, x: 0, transition: springGentle },
};

// ── Toast / notification ──
export const toastVariants: Variants = {
  initial: { opacity: 0, y: -24, scale: 0.92 },
  animate: {
    opacity: 1,
    y: 0,
    scale: 1,
    transition: springSnappy,
  },
  exit: { opacity: 0, y: -12, scale: 0.92, transition: { duration: 0.2 } },
};

// ── Modal ──
export const modalBackdrop: Variants = {
  initial: { opacity: 0 },
  animate: { opacity: 1, transition: { duration: 0.25 } },
  exit: { opacity: 0, transition: { duration: 0.2 } },
};

export const modalContent: Variants = {
  initial: { opacity: 0, scale: 0.92, y: 20 },
  animate: { opacity: 1, scale: 1, y: 0, transition: springGentle },
  exit: {
    opacity: 0,
    scale: 0.95,
    y: 10,
    transition: { duration: 0.2 },
  },
};
