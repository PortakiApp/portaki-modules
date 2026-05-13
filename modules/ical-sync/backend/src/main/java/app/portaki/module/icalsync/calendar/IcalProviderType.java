package app.portaki.module.icalsync.calendar;

/**
 * Famille de flux iCal (Airbnb a des particularités de contenu ; le générique couvre le RFC 5545
 * basique extrait par le parseur interne).
 */
public enum IcalProviderType {
    AIRBNB,
    GENERIC
}
