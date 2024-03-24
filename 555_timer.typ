/*
 En inre blick av 555-kretsen © 2023 by Tobias Per Leopold Wennberg is licensed under CC BY-SA 4.0. To view a copy of this license, visit http://creativecommons.org/licenses/by-sa/4.0/
*/
#import "template.typst": conf

#let Vcc=text[`V`#sub[`cc`]]
#let TRIG=overline[`TRIGGER`]
#let Vcontrol=text[`V`#sub[`CONTROL`]]

#let title = [
En inre blick av 555-kretsen
]
#show: doc => conf(
title: title,
authors: (
   (
      name: "Tobias Per Leopold Wennberg",
	 affiliation: "Stockholm Science & Innovation School (SSIS)",
	 email: "tobias.wenn@pm.me",
  ),
),
doc,
)


= Om
En 555 timer integrerad krets (IC) är en typisk krets för att skapa timer funktioner. Dessa kan vara förskjutningar, eller pulsgenerering. Pinouten är följande:
+ `GND`
+ $Vcc$
+ $TRIG$
+ `OUTPUT`
+ #overline[`RESET`]
+ `CONTROL`
+ `THRESHOLD`
+ `DISCHARGE`


== `GND`
_Ground/jord_ Detta är jord och ska (oftast) kopplas till motsvarande jord i kretsen. 

== `Vcc`
_Voltage Common (c)_ ska kopplas till ström.

== *$TRIG$*
När $TRIG$ faller lägre än $1/2$ av $Vcontrol$ blir `OUTPUT` hög och timer intervallen startar. Medan $TRIG$ fortsätter vara låg är `OUTPUT` hög.

== `OUTPUT`
_push-pull_ kontakten (p.p) driver mellan låg och hög. 

== #overline[`RESET`]
Timinginterval återställs när denna blir låg.

== `CONTROL`
Ger tillgång till interna spänningsdivideraren. Kopplas oftast till en $470 mu F$ kondensator.

== `THRESHOLD`
När denna är högre än $Vcontrol$ slutar `OUTPUT` hög intervallen och `OUTPUT` blir låg.

== `DISCHARGE`
Detta är en _open-drain (O.D)_ port. Den kopplas till `GND` när `OUTPUT` är låg.


#show: rest => columns(1, rest)
= INRE DELAR
En 555-timer's krets är uppdelad av 6 delar:
+ Voltage divider
+ `THRESHOLD` jämnförare
+ $TRIG$ jämnförare
+ Latch
+ Output
+ Discharge

== VOLTAGE DIVIDER
Härifrån kommer namnet 555 i 555-timer. Det är tre $5 k Omega$ motstånd seriekopplade mellan jord och spänning:

#align(center)[#image("./voltage_divider.svg", height: 3cm)]

Detta ger en spänning av $1/3 Vcc$ vid $"VD"_1$ och $2/3 Vcc$ vid $"VD"_2$

== `THRESHOLD-` OCH `TRIGGER` JÄMFÖRARE
En jämförare jämför två spänningar. Den ger hög om $V_+ > V_-$ och låg om $V_+ < V_-$

$ 
	V_("out") := cases(
		1 "om" V_(+) > V_(-),
		0 "om" V_{+} < V_(-),
	)
$

Det är två jämförare i 555'an. Threshold jämföraren jämför $"VD"_1$ (från `Voltage Divider`) och `THRESHOLD`; trigger jämföraren jämför $"VD"_2$ med $TRIG$.

#align(center)[#image("comparator.svg", height: 3cm)]


== LATCH
555'an använder en set-release (SR) latch som sparar statusen på timern. En SR-latch bevarar senaste inputen: om senast _S_ var hög är _Q_ hög och _#overline("Q")_ låg; om senast _R_ var hög är _#overline("Q")_ hög och _Q_ låg. Många latch'ar har även en _RESET_ som tar _Q_ låg och _#overline("Q")_ hög. Vissa SR-latch'ar har inte _#overline("Q")_ och på 555'an är den oanvänd. Latchen är kopplad mellan C#sub("TH") och C#sub("TR") från jämförarna. 

#align(center)[#image("latch.svg", height: 3cm)]

== OUTPUT
Denna förstärker *inversen* av signalen från `Latch`.

== DISCHARGE
Kopplar `DISCHARGE` till `GND` när `OUTPUT/Latch` är hög.

#align(center)[#image("discharge.svg", height: 3cm)]

= RESULTAT
#align(center)[#image("full.svg", height: 4cm)]

= LÄGEN
Man kan koppla in 555 kretsen på många sätt. Dessa sätt kallas lägen. Beroende på läget fungerar kretsen på olika sätt. De främsta lägena är
- Astabil
- Monostabil
- Schmitt trigger
Några andra noterbara är
- *Sågtands oscillatorn* som skapar vågor gradvis och sedan dyker, likt ett sågblad.
- *Låg sändninscykel oscillatorn*, som skapar en kort hög puls.
- *Pulsbredds modulatorn*
- *Förlorad puls detektorn*, som märker om en puls inte kom.


== ASTABIL
#align(center)[ #image("astabil_circuit.svg", width: 6cm)]
i astabilt läge ger 555'an en kontinuerlig rektangulär puls med en satt period.

#align(center)[#table(
columns: (auto, auto),
inset: 0pt,
gutter: 0pt,
align: horizon,
fill: none,
stroke: none,
[#image("astabil_pulse.svg", height: 5.8cm)], [
     #table(
     columns: (auto, auto, auto),
    inset: 10pt,
    align: horizon,
    [*C spänning*], [*inre latch*], [`OUTPUT`],
        $0$, "0", "1",
        $2/3 Vcc$, "0", "1",
        $1/3 Vcc$, "1", "0",
        $2/3 Vcc$, "0", "1",
        $1/3 Vcc$, "1", "0",
     )
]
)]

+ C är 0V, vilken innebär att `TRIGGER` är $< 1/3  Vcc$ och `OUTPUT` = 1. Det innebär även att `DISCHARGE` är öppen och kondensatorn laddas.
+ C är > $2/3 Vcc$ vilket innebär `DISCHARGE` $> 2/3 Vcc$ och `OUTPUT` = 0. Det innebär även att `DISCHARGE` är stängd och kondensatorn börjar laddas ur.
+ C är < $1/3 Vcc$ vilket innebär att `DISCHARGE` är $< 1/3 Vcc$  och `OUTPUT` = 1. `DISCHARGE` öppnas och kondensatorn börjar laddas.


Notera att laddningen går genom R#sub[1] och R#sub[2] medan urladdningen endast går genom R#sub[2]. Detta innebär att `OUTPUT` kommer vara hög längre än låg. Ekvationen är

$ t_("high") &= ln 2 dot (R_1 + R_2)C \
 t_("low") &= ln 2 dot R_2 C $

och frekvensen $f$ blir 

$ f = 1/t_("high") + t_("low") = 1/(ln 2 dot (R_1 + 2R_2)C) $

och sändningscykeln \(D\) blir

$ D("%") = t_("high")/t_("high") + t_("flow") 100 = (R_1 + R_2)/(R_1+2R_2) dot 100 $

=== HÄRLEDNING FORMLER
#align(center)[#image("RC_charging.svg", height: 3cm)]
För en RC laddningskrets, där man laddar en kondensator över ett motstånd, tar det en viss tid för kondensatorn att ladda. Denna tid kommer bero på motståndet - _R_ - och kondensatorns - _C_.  

För fallande läget av spänningen över kondensatorn gäller differentialekvationen

 $ C (d V)/(d t) + V/R = 0 $
 #table(
 columns: (auto,auto),inset: 0pt, gutter: 0pt, align: horizon, fill:none, stroke: none,
 [
 Där V är spänningen över kondensatorn, C kapacitansen och R motståndet mellan kondensatorn positiva och negativa sida.
 ],[ #align(center)[#image("RC_discharge.svg", height: 2cm)]])
 #let IF=$e^(t/(R C))$
 #let nIF=$e^(-t/(R C))$
 $
 (d V)/(d t) + V /(R C) &= 0 \
 #text("I.F.") & IF \
 d/(d t) V IF &= 0 \
 V IF &= K#text(", där K är en konstant") \
 V &= K nIF
 $

Den allmäna formel för en kondensators naturliga urladdning i en RC krets är därmed $V = K e^(-t/(R C))$. Vi kan nu applicera den för bistabila 555-kretsen:

 $
 V &= K e^(-t/(R C)) \
 V(0) &= 2/3 epsilon#text(", där") epsilon#text(" är spänningen i kretsen") \
 K &= 2/3 epsilon \
 V &= 2/3 epsilon e^(-t/(R C)) \
 V(t_1) &= epsilon/3 \
 2/3 epsilon e^(-t_1/(R C)) &= epsilon/3 \
 e^(-t_1/(R C)) &= 1/2 \
 -t_1/(R C) &= ln 1/2  \
 t_1 &= ln 2 dot R C 
 $

 En allmän funktion för spänningsändringen för en kondensator i en RC-krets är lite mer komplicerad att härleda. Enligt _Kirchoff's lag_ vet vi $epsilon - V_R -V_c = 0$ där $epsilon$ är kretsens spänning, $V_R$ är spänningen över motståndet och $V_C$ spänningen över kondensatorn i våran krets där resistorn och kondensatorn är seriekopplad mellan spänningskällan och jord. Kapacitans är definerad $C=q/U equiv V_C = q/C$. Med _Ohm's lag_ vet vi $V_R = I R$ och ströms definition vet vi $I = (d q)/(d t)$. 

 $
 epsilon - V_R - V_C &= 0 \
 epsilon - I R - q/C &= 0 \
 epsilon - R (d q)/(d t) -  q/C &= 0
 $

 Vi löser diff. ekvationen
 #let IF=$e^(t/(C R))$
 #let nIF=$e^(-t/(C R))$
 $
 (d q)/(d t) + q/(C R) &= epsilon/R  \
 #text("I.F. ") &IF \
  d/(d t)q IF &= epsilon/R IF \
 q IF &= integral epsilon/R IF d t = C epsilon dot IF + K", Där K är en konstant" \
 q &= C epsilon + K nIF 
 $

 Om kretsen startar på $0C$ säger vi

 $
 q(0) &= 0 \
 C epsilon + K e^(-0/(C R)) &= 0 \
 C epsilon + K &= 0 \
 K &= -C epsilon \
 q &= C epsilon - C epsilon dot nIF \ &= C epsilon (1 - nIF) \
 $

 För överiga värden på startladdningen $q_0$ räknas
 
 $
 q(0) &= q_0 \
 C epsilon + K nIF &= q_0 \
 C epsilon + K &= q_0 \
 K &= q_0 - C epsilon \
 q &= C epsilon + (q_0 - C epsilon) nIF
 $

 Nu kan vi skapa en funktion för spänningen
 $ V_c=q/C = (C epsilon + (q_0 - C epsilon) nIF)/C = epsilon + (q_0 - C epsilon)/C nIF) = epsilon + (V_0 - epsilon)nIF $
 Med denna formel kan vi härleda båda faserna för astabila läget.

 Vi sätter $R_1$ och $R_2$ som motstånden och $C_1$ kondensatorn i våran 555'krets. För fas 1 löser vi t:

 $
 R &= R_1 +R_2 \
 C &= C_1 \
 V_0 &= 1/3 epsilon \
 V_c (t) &= 2/3 epsilon \
 epsilon + (1/3 epsilon - epsilon)nIF &= 2/3 epsilon \
 3/2 + 1/2 - 3/2 nIF &=1 \
 nIF &= 1/2 \
 t/(C R) &= ln 2 \
 t = ln 2 dot C R &= ln 2 dot C_1(R_1 + R_2)
 $

 För fas 2:
 $
 R &= R_2 \
 C &= C_1 \
 V_0 &= 2/3 epsilon \
 V_c (t) &= 1/3 epsilon \
 epsilon + (2/3 epsilon - epsilon) nIF &= 1/3 epsilon \
 3 + (2 - 3)e^(-t/( C R)) &=1 \
 nIF &= 2 \
 t/(C R) &= ln 2 \
 t = ln 2 dot C R &= ln 2 dot C R_2 
 $

== MONOSTABIL
I monostabilt läge produceras en hög puls under en viss tid, beroende på R och C när `TRIG` ger en puls $> 1/3Vcc$ (_hög_). `TRIG`'s normalläge ska vara $< 1/3Vcc$(_låg_). 

#align(center)[#image("monostabil_circuit.svg", height: 5cm)]
+ `TRIG` börjar låg. `THRESHOLD`$> 2/3 Vcc$ vilket innebär att `OUTPUT` = 0 och `DISCHARGE` är öppen. $V_C arrow 0$
+ `TRIG` ger en puls. Latchen slår om, `OUTPUT` blir hög, `DISCHARGE` stängs, och kondensatorn börjar ladda genom motståndet.
+ $V_C > 2/3 Vcc$. Latchen slår om, `OUTPUT` blir låg, `DISCHARGE` öppnas, och kondensatorn laddar ur snabbt.
Tiden för att ladda motståndet är $t = R C ln 3$

=== HÄRLEDNING FORMEL
Vi vet från härledningen för bistabil att tiden för spänningsförändringen för kondensatorn i en RC-krets är 
$ V_c &= epsilon + (V_0 - epsilon)nIF $
Spänningen i kondensatorn vid start är försumbar, och den är klar vid $V_C = 2/3 Vcc$.

$
V_c &= epsilon + (V_0 - epsilon)nIF \
V_0 &= 0 \
V_c &= epsilon - epsilon nIF = epsilon (1 - nIF) \
V_c (t_1) &= 2/3 epsilon \
epsilon (1 - e^(-t_1/(C R))) &= 2/3 epsilon \
1 - nIF &= 2/3 \
nIF &= 1/3 \
t &= C R ln 3
$

== BISTABIL SR LATCH
555'an kan användas som en aktiv-låg SR latch

#align(center)[#image("SR_latch.svg", height: 5cm)]
Hur denna krets fungerar är trivialt och lämnas som uppgift åt läsaren.
