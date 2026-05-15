package dev.cyrilcolinet.portaki.module.trmnl.model;

import java.util.Map;

/**
 * Enveloppe des variables fusionnées envoyées à TRMNL ({@code merge_variables}).
 */
public record TrmnlPayload(Map<String, Object> mergeVariables) {

}
