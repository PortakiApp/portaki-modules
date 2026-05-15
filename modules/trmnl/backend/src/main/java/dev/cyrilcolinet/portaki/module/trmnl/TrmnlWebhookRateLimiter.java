package dev.cyrilcolinet.portaki.module.trmnl;

import java.time.Duration;
import java.time.Instant;
import java.util.ArrayDeque;
import java.util.Deque;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Limite les appels webhook TRMNL (plan gratuit ~12/h, marge conservée : 11/h par URL).
 */
public final class TrmnlWebhookRateLimiter {

    private static final int MAX_PER_WINDOW = 11;
    private static final Duration WINDOW = Duration.ofHours(1);

    private final Map<String, Deque<Instant>> hitsByKey = new ConcurrentHashMap<>();

    public boolean tryAcquire(String rateLimitKey) {
        Instant now = Instant.now();
        Instant cutoff = now.minus(WINDOW);
        Deque<Instant> q = hitsByKey.computeIfAbsent(rateLimitKey, k -> new ArrayDeque<>());
        synchronized (q) {
            while (!q.isEmpty() && q.peekFirst().isBefore(cutoff)) {
                q.pollFirst();
            }
            if (q.size() >= MAX_PER_WINDOW) {
                return false;
            }
            q.addLast(now);
            return true;
        }
    }
}
