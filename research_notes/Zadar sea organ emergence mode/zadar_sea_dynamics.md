# Ocean Forcing of the Zadar Sea Organ: Waves, Wakes, Tides, Wind, and Resulting Temporal Dynamics

Scope note: Zadar-specific numbers are given where found. Where only general ocean-wave or ship-wake theory was available, the item is labelled **[GENERAL]**. Main Zadar-specific primary sources:
(1) Krvavica, Peroli, Ružić & Ožanić, "Time-dependent numerical model for simulating internal oscillations in a sea organ". The arXiv preprint is 1903.09314 and the paper was published in Ocean Engineering in 2020, DOI 10.1016/j.oceaneng.2020.107336.
(2) The Croatian Hydrographic Institute (HHI) official tide tables, "Mijene 2026".

## 1. Wave climate at the Zadar riva (Hs, periods, sheltering, seasonality, bura/jugo/maestral)

### Takeaway
No published wave-buoy climatology exists for the Zadar riva (Istarska obala at the tip of the peninsula, facing the Zadar Channel toward Ugljan). The best Zadar-specific values come from the Krvavica et al. Sea Organ model. It uses two design sea states from the Adriatic Tabain spectrum:
- **Typical, light NW wind (maestral-type):** Hs ≈ 0.4 m, Tp ≈ 2.9 s.
- **Strong SE wind (jugo):** Hs ≈ 1.0 m, Tp ≈ 4.5 s.

These two cases are good anchors for a "calm/typical" state and a "storm" state. Adriatic waves are fetch-limited and short-period. Jugo produces higher and longer waves than bura at the same wind speed because it has a longer fetch.

### Cited Findings
- The Sea Organ model was forced with two irregular wave conditions generated from the Tabain (Adriatic) spectrum. Case 1 had Hs = 0.4 m from a light northwest wind. Case 2 had Hs = 1.0 m from a strong southeast wind. Still-water level was +0.35 m asl, and 15-minute wave fields were simulated. — [Krvavica et al., arXiv:1903.09314 / Ocean Eng. 2020](https://arxiv.org/abs/1903.09314)
- The peak frequency was fp = 0.34 Hz for Case 1 (Tp ≈ 2.9 s) and fp = 0.22 Hz for Case 2 (Tp ≈ 4.5 s). — [Krvavica et al.](https://arxiv.org/abs/1903.09314)
- Krvavica et al. examined the wave climate on the paper's Fig. 7 at 60-second resolution. Surface elevation excursions in the figures were roughly ±25 cm for Case 1 and ±50–100 cm for Case 2. These values are read from plotted axes, so they are approximate. — [Krvavica et al.](https://arxiv.org/abs/1903.09314)
- The Tabain spectrum is a single-parameter modification of JONSWAP, based on limited wave measurements and ship observations. It is "generally accepted for marine technology problems in the Adriatic Sea". — [Katalinić & Parunov, JMSE 2023 (MDPI 11/1/42)](https://www.mdpi.com/2077-1312/11/1/42); [Katalinić PhD thesis, FSB Zagreb](https://www.fsb.unizg.hr/atlantis/web/sites/fsbonline/newsboard/372/31050/Doktorski_rad__M_Katalinic.pdf)
- Because the Adriatic is narrow, waves are fetch-limited and usually not very large. Sirocco (jugo) blows along the basin's long axis and bura across its short axis. Sirocco therefore produces much higher waves than bura at the same wind speed and duration. — [JMSE / DOAJ summary of Adriatic wave papers](https://doaj.org/article/255056dc1e7d4cd0bac5b020245e69e2); [Frontiers in Climate 2024](https://www.frontiersin.org/articles/10.3389/fclim.2024.1409237/full)
- Open-Adriatic extremes are much larger than conditions at the Zadar riva. For example, a Dubrovnik buoy recorded Hs = 4.75 m and Hmax = 10.87 m during a southerly storm. This is not representative of the sheltered Zadar Channel. — [Swellnet 2019](https://swellnet.com/news/swellnet-analysis/2019/11/15/largest-wave-ever-recorded-in-the-adriatic-sea-near-dubrovnik)
- Maestral is a summer afternoon sea breeze. It builds from around 10:00–11:00, blows from WNW, and reaches about 12–22 kn by mid-afternoon. Jugo builds gradually over several days with a "distinct south-easterly swell" and is most common in autumn. Bura brings sudden NE gusts above 40 kn to the northern Zadar islands. These descriptions come from commercial charter guides, so they are not authoritative. — [Charter guide (Zadar)](https://charter-compass-pro.lovable.app/journal/zadar-yacht-charter-cruising-guide); [Croatian weather article on maestral](https://vreme.com/?p=4558353)
- No wave measurements exist for the comparable Vodice coast, about 40 km SE. There, wave climate was derived indirectly by hindcast from a 41-year Šibenik wind record. This suggests no measured local wave climatology is available for this coast segment either. — [Vjetrovalna klima Vodice (consultancy report)](https://grad-vodice.hr/assets/nabava/files/bagatelna-nabava/2015/19-03-2015/vjetrovalna-klima-vodice_tekst.pdf)
- Current measurements in the Zadarski and Pašmanski channels have been used to study the channel's response to different wind forcing. These are currents only, with no waves. — [CROSBI 33190](https://www.bib.irb.hr/33190)

### Inferences
- **Geometry (inference):** The organ faces roughly W/NW into the Zadar Channel. Ugljan is a few km across, so fetch toward the W/SW is short. The longest fetches run along the channel axis, toward NW (maestral) and toward SE (jugo). Bura (NE) blows offshore from the mainland at the riva, so it should generate very little local wave energy directly at the organ. The "loud during bura" claim is therefore doubtful, except through gust-driven chop, channel swell, or seiche/sea-level effects. Jugo and maestral are the wind regimes most likely to drive strong activity. This is consistent with Krvavica et al. choosing NW for typical conditions and SE for strong conditions.
- **Proposed parameter set for a generative model.** These are calibrated from the Krvavica cases. The other states are interpolation and inference.

  | State | Hs (m) | Tp (s) | Notes |
  |---|---|---|---|
  | Calm / glassy (common on summer mornings and in winter anticyclones) | 0.05–0.15 | 2–3 | Activity mostly from wakes |
  | Light maestral (summer afternoons) | 0.2–0.5 | 2.5–3.2 | Anchor: 0.4 m / 2.9 s |
  | Strong jugo (autumn and winter, multi-day) | 0.8–1.2+ | 4–5 | Anchor: 1.0 m / 4.5 s |
  | Bura (winter, gusty) | Low to moderate at the riva | 2–3 | Short, steep chop; offshore wind |

- **Diurnal cycle (summer):** quiet morning, then maestral ramp-up from about 10–11 h, a peak at about 14–17 h, and decay in the evening. **Seasonal cycle:** autumn and winter bring multi-day jugo episodes (lasting 1–3 days) separated by calmer periods.

### Gaps
- No measured Hs/Tp climatology (buoy or ADCP) was found for the Zadar Channel or the riva. No exceedance statistics by month were found either.
- The Tabain Tp(Hs) formula coefficients could not be retrieved, because the MDPI page returned 403.
- The paper "A Simplified Computational Model of the Sea Organ in Zadar" (Zbornik radova, Građevinski fakultet Rijeka) includes a local wind-wave climate analysis. Its full text was unreachable (redirect loop). — [Zbornik radova record](https://zr.gradri.uniri.hr/zr/en/article/view/7)
- No frequency-of-occurrence data for maestral, jugo, and bura at Zadar (DHMZ) was retrieved.

## 2. Ship and ferry wakes

### Takeaway
The authors of the main hydraulic paper confirm that passing boats are one of the three forcings of the organ, alongside waves and tides. Zadar's car ferries now use Gaženica, about 4 km SE, so fewer of them pass the riva. Passenger ships and catamarans still use the old-town port (Liburnska obala), on the NE/harbour side of the peninsula. They plus private and excursion boats pass close to the organ when entering or leaving the Zadar Channel. General wake literature shows that wakes arrive as discrete packets of a few to about 10–20 longer-period waves. These would appear as bursts lasting tens of seconds on top of the background sea.

### Cited Findings
- "The waves, tides, and passing boats initiate the movement of the sea surface…" — [Krvavica et al.](https://arxiv.org/abs/1903.09314)
- All Zadar car ferries operate from the Gaženica terminal, 4 km SE of the old town. — [In Your Pocket Zadar](https://inyourpocket.com/zadar/arriving)
- The old port (Liburnska obala 4) remains in use for passenger-only ferries and catamarans to destinations such as Pula, Rijeka, Mali Lošinj, and Silba. Sources disagree on exact route assignments. — [Ferryhopper](https://cdn1.ferryhopper.com/en/destinations/croatia/zadar); [Misterferry](https://www.misterferry.com/en/ports/ferry-zadar.aspx); [Direct Ferries](https://www.directferries.de/zadar_faehre.htm)
- The Zadar–Preko route (line 431 car ferry from Gaženica, plus a passenger line from the old town) runs 25–29 crossings per day in high season and 17–24 in low season. Another source gives up to 29 in July–August, about 25 in June and September, and up to 20 for the rest of the year. Departures run from about 01:00 to 23:59. — [Croatiaferries](https://croatiaferries.com/preko-ferry-port.htm); [Ferrycroatia](https://ferrycroatia.com/en/zadar-preko/); [Jadrolinija timetable PDF](https://www.jadrolinija.hr/download/4d44a1e3eb6b8b5d6122714d81a3f8ed)
- **[GENERAL]** Fast-ferry wakes have unusually long periods and sometimes large amplitudes. They act as a new forcing on coasts not exposed to ocean swell. — [Torsvik et al., NPG 2009](https://npg.copernicus.org/articles/16/351/2009/)
- **[GENERAL]** In Tallinn Bay, the highest fast-ferry wake components often had periods of about 10–15 s, which do not occur naturally in that area. — [Kurennoy & Soomere](https://coruna.coastdyn.ru/iccal/pub/kurennoy.pdf)
- **[GENERAL]** When a catamaran accelerates (for example, leaving port), the period of its largest wake wave can grow by more than 30% over the steady-state value. — [Macfarlane et al., AMC](https://www.amcsearch.com.au/source-assets/images/GJM_KGP_MC-Wave-Wake-during-Acceleration-AUTHOR-VERSION.pdf)
- **[GENERAL]** Wake-energy prediction tools underpredict the most energetic waves near depth Froude number ≈ 1.0. — [Macfarlane et al. CP2023](https://www.amcsearch.com.au/source-assets/images/CP2023-Macfarlane-26June2023.pdf)

### Inferences
- **Wake period vs organ resonance:** the organ's segment resonances are about 1.9–4.7 s (see section 5). Wakes from small and medium vessels in a channel usually have periods of about 2–5 s, which would couple efficiently. Long-period fast-ferry leading waves (8–15 s) would cause slow, large surges with strong water-level (rather than pressure) response. This is inference that combines general wake physics with the Krvavica resonance table.
- **Model suggestion:** Treat wakes as a Poisson process.
  - **Rate:**
    - Summer daytime: about 4–10 per hour (catamarans, excursion boats, yachts, island passenger ships).
    - Winter: about 1–3 per hour.
    - Night (00–06 h): close to 0.
  - **Each event:** a packet of 5–15 waves. Height is 0.2–0.6 m, period 2–5 s for typical vessels (occasionally 6–10 s leading waves). The packet lasts 20–60 s, with a crescendo-then-decrescendo envelope. The leading longer waves are followed by shorter, higher-frequency ones (dispersive Kelvin wake).
  - These rates are estimates only. No traffic count past the riva was found.

### Gaps
- No vessel counts (AIS-based) or wake measurements at the Zadar riva were found.
- No source confirming "louder during boat passages" from observers was found, beyond the authors' statement listing boats as a forcing.

## 3. Tides at Zadar

### Takeaway
Zadar is strongly microtidal, with a mixed tide that is mainly diurnal. Predicted daily ranges are about 8–32 cm (mean about 26 cm in September 2026). The theoretical maximum astronomic range is about 0.5–0.6 m. Meteorological effects (pressure up to about 30 cm, wind setup) are comparable to the tide itself. Tide matters for the organ: the seven segments' inlet/step crest elevations step from 0.3 to 1.3 m asl, and the modelling assumed a still-water level of +0.35 m asl.

### Cited Findings
- HHI harmonic constants for Standard Port Zadar (44°07'N, 15°14'E) use 18.6 years of data centred on 2003.5. Z0 = 23 cm above chart datum. Amplitudes H (cm):

  | Constituent | Amplitude (cm) |
  |---|---|
  | M2 | 5.76 |
  | S2 | 3.06 |
  | N2 | 1.08 |
  | K2 | 0.74 |
  | K1 | 11.40 |
  | O1 | 3.27 |
  | P1 | 3.98 |

  — [HHI, Mijene 2026 (tide tables), Table 1](https://www.hhi.hr/Portals/0/adam/HHI/pMB-Mvl4nUWpnu4mb5ShNw/PdfView/Mijene%202026%20za%20web.pdf)
- East Adriatic tides are of mixed type with pronounced inequalities in height. Low pressure raises sea level and high pressure lowers it; pressure effects seldom exceed 30 cm. Onshore winds raise high waters, and offshore winds have the reverse effect. — [HHI Mijene 2026](https://www.hhi.hr/Portals/0/adam/HHI/pMB-Mvl4nUWpnu4mb5ShNw/PdfView/Mijene%202026%20za%20web.pdf)
- The web edition contains a September 2026 Zadar sample page. I parsed it myself:
  - Predicted heights range from 2 to 41 cm above chart datum.
  - The daily range (max minus min) is 8–32 cm, with mean 25.6 cm and median 26 cm.
  - Of 30 days, 18 had 4 extremes per day, 5 had 3, and 7 had only 2, meaning diurnal-only days around the tropic/neap phase. Example: 4–6 Sept show a single high near 35–37 cm and a single low near 4–8 cm.
  
  — [HHI Mijene 2026](https://www.hhi.hr/Portals/0/adam/HHI/pMB-Mvl4nUWpnu4mb5ShNw/PdfView/Mijene%202026%20za%20web.pdf)
- The semidiurnal amphidrome lies between Ancona and Šibenik, so semidiurnal amplitudes are small in the central Adriatic, including Zadar. The M2 maximum is about 26 cm in the Gulf of Trieste. At Zadar, the diurnal OO1 harmonic (3.8 cm) exceeds O1, Q1, S2, N2 and K2. Note that HHI's O1 = 3.27 and S2 = 3.06 are close to this, and the different analyses do not fully agree. — [HHI Adriatic Sea Level project](https://www.hhi.hr/en/about-us/projects/adriatic-sea-level); [EGU2017-13236 abstract](https://meetingorganizer.copernicus.org/EGU2017/EGU2017-13236.pdf?pdf)
- Estimated segment geometry of the Sea Organ (middle pipe per segment):
  - Crest/step elevation hc = 0.30, 0.55, 0.70, 0.85, 1.00, 1.16, 1.30 m asl for segments 1–7.
  - The pipe inlet depth is h = −0.2 m asl for all segments.
  - Horizontal entry length L1 = 2.9, 2.9, 2.9, 1, 1, 1, 0.5 m.
  - Sloped pipe length L2 = 5.26, 4.26, 3.60, 4.26, 3.60, 2.55, 2.20 m.
  - Slope φ = 20, 25, 30, 25, 30, 45, 55°.
  - Entry diameter D1 = 0.3 m; D2 = D3 = 0.075 m; labium orifice A0 = 112 mm².
  
  — [Krvavica et al., Table 1](https://arxiv.org/abs/1903.09314)
- The wave reflection coefficient at the organ wall depends on crest height hc above still-water level relative to Hs (Goda 2000 formulation). This makes response a function of water level. — [Krvavica et al.](https://arxiv.org/abs/1903.09314)

### Inferences
- **Form factor:** F = (K1+O1)/(M2+S2) = (11.40+3.27)/(5.76+3.06) ≈ 1.66. This classifies the tide as mixed, mainly diurnal (1.5 < F < 3).
  - **Dominant periods:** K1 23.93 h, O1 25.82 h, P1 24.07 h, M2 12.42 h, S2 12.00 h. **[GENERAL]** constituent periods.
  - **Spring–neap / tropic modulation:** about 13.66 days for the diurnal tropic cycle (K1–O1 beat) and about 14.77 days for the semidiurnal spring–neap cycle (M2–S2 beat).
  - The sum of the seven amplitudes is about 29 cm, so the maximum astronomic range is about 58 cm.
- **Model suggestion:** Represent sea level as η_tide(t) = Σ H_i cos(ω_i t − g_i), using HHI constants and phases (g for Zadar: M2 221.6°, S2 220.5°, N2 228.7°, K2 199.6°, K1 52.9°, O1 53.6°, P1 53.7°). Add a slow meteorological residual of ±10–30 cm with a time scale of hours to days. Jugo plus low pressure gives a positive residual; bura plus high pressure gives a negative residual.
- **Effect on activity (inference from geometry):**
  - Higher water reduces the freeboard hc − SWL. This increases wave overtopping and pressure at the inlets, and changes reflection.
  - Segment 1 is lowest (0.3 m asl), so it is the most sensitive to water level.
  - The tide changes still water by about ±15 cm around a mean of about 0.2–0.35 m asl, so expect a modest (perhaps ±10–30%) modulation of mean activity over a 12–25 h cycle.
  - Inlets sit at −0.2 m. Krvavica's still-water level of +0.35 m implies about 0.55 m submergence. At low water (about 0 m asl) submergence drops to about 0.2 m, and big wave troughs might uncover inlets (air gulping, sputtering). This is speculative.
- **Additional low-frequency forcing [GENERAL Adriatic knowledge, not sourced here]:** Adriatic seiches (fundamental period about 21–22 h) can add several to tens of cm during storms. Treat this as part of the meteorological residual.

### Gaps
- No direct measurement relates organ loudness to tidal stage.
- The chart datum to asl (HVRS71) offset at Zadar was not found. Mixing "cm above chart datum" (tide tables) with "m asl" (Krvavica geometry) needs an offset (inferred about 0.2–0.3 m, unverified).
- Only one month of the 2026 Zadar predictions is in the web excerpt.

## 4. Wave group statistics (sets, spectra, quiet gaps)

### Takeaway
Wind-sea grouping theory is general, not Zadar-specific, but it gives usable parameters. Use a JONSWAP/Tabain spectrum with random phases to synthesize the surface (as Krvavica et al. did). Grouping then emerges naturally. For an event-based model, use Goda/Kimura run-length statistics. Short-period fetch-limited seas (Tp about 3–4.5 s) give "sets" only tens of seconds long, so quiet gaps are short. Long silences occur only when Hs itself is small or between wakes.

### Cited Findings
- Krvavica et al. synthesized the sea surface as a sum of harmonics. Amplitudes are a_i = sqrt(2 S(ω_i) Δω) from a Pierson–Moskowitz or JONSWAP spectrum (they used the Tabain spectrum), with uniform random phases. Reflection is applied as (1 + Kr). — [Krvavica et al.](https://arxiv.org/abs/1903.09314)
- **[GENERAL]** Goda (1970) defined the "run of high waves" (consecutive waves above a threshold such as H1/3) and the total run length, and introduced the spectral peakedness parameter Qp. Goda's independence assumption systematically underpredicts observed group lengths. Kimura's Markov-chain model makes mean group length depend on the correlation coefficient between successive wave heights. Battjes & van Vledder (1984) derived that coefficient from the spectrum via envelope theory. — [ICCE proceedings](https://icce-ojs-tamu.tdl.org/icce/article/download/3824/3507/16184); [Medina & Hudspeth 1990](https://data-ww3.ifremer.fr/BIB/Medina_Hudspeth_CE1990.pdf)
- **[GENERAL]** The groupiness factor (GF) is computed from the SIWEH (smoothed instantaneous wave energy history), using a Bartlett filter of width 2·Tp. Finite-record scatter is large enough that mean group length correlates poorly with spectral shape. — [HR Wallingford SR39](https://eprints.hrwallingford.com/80/1/SR39.pdf); [Masson & Chandler 1993](https://data-ww3.ifremer.fr/BIB/Masson_Chandler_CE1993.pdf)

### Inferences
These values are **[GENERAL]** theory applied to the Zadar Tp values above.
- **Groups and sets:** For JONSWAP with γ ≈ 3.3, the correlation between successive wave heights is about 0.4–0.6. The mean run length of waves above H1/3 is about 1.5–2 waves, and the total run (group repetition) is about 5–8 waves.
  - With Tp ≈ 3 s, this gives "set" durations of about 5–6 s and set-to-set intervals of about 15–25 s.
  - With Tp ≈ 4.5 s (jugo), it gives sets of about 7–9 s every 25–40 s.
- **Individual wave heights:** Rayleigh distributed with H_rms = Hs/√2. The fraction of waves exceeding a pipe-activation threshold H* is exp(−2(H*/Hs)²). This gives a simple, physically based "notes per minute" function of Hs.
  - Example: if H* ≈ 0.3 m, P(exceed) ≈ 0.6% at Hs = 0.2 m, about 32% at Hs = 0.4 m, and about 84% at Hs = 1.0 m.
  - H* is not measured; it is a calibration parameter for the generative model.
- **Wave-to-wave intervals:** zero-crossing period Tz ≈ Tp/1.28 for JONSWAP, giving about 2.3 s (calm/maestral) and about 3.5 s (jugo).
- **Layered model architecture:**
  - Slow layer, hours to days: weather-state Markov chain for Hs/Tp regime (calm/maestral/jugo/bura), driven by the diurnal maestral cycle and multi-day jugo episodes.
  - Tide layer: harmonic water level.
  - Wake layer: Poisson packets.
  - Fast layer, seconds: spectral synthesis of the surface, which produces groups.

### Gaps
- No Zadar wave time-series exists from which to estimate local groupiness, correlation coefficients, or spectral γ.
- The Tabain γ and fp(Hs) coefficients were not retrieved.

## 5. Observed and modelled behaviour of the organ in relation to sea state

### Takeaway
The structure is about 70–75 m long, with 7 segments × 5 pipes = 35 pipes. Each segment is a small oscillating-water-column resonator with a natural period of about 1.9–4.7 s. Segment 1 is the slowest and segment 7 the fastest. Each segment responds most strongly when the wave peak period matches its resonance. Small waves (Hs 0.4 m) excite the middle segments (2–5) and leave segment 1 quiet or silent. Big jugo waves (Hs 1.0 m) wake up segment 1 and favour segment 4, while segments 6–7 become relatively weaker. Visitors report near-inaudibility on calm days and louder, deeper sound with larger waves.

### Cited Findings
- **Layout and tuning:** 75 m long, seven segments, five pipes each. Odd segments are tuned to G-major tones (D-G-d-g-h) and even segments to C-major with added sixth (C-D-c-e-a). All tones fall within 60–250 Hz. — [Krvavica et al.](https://arxiv.org/abs/1903.09314)
- **Resonance by segment** (natural fn and resonance fr, in Hz):

  | Segment | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
  |---|---|---|---|---|---|---|---|
  | fn (Hz) | 0.218 | 0.266 | 0.311 | 0.277 | 0.327 | 0.457 | 0.538 |
  | fr (Hz) | 0.215 | 0.263 | 0.308 | 0.275 | 0.323 | 0.449 | 0.528 |
  | Resonant period (s) | 4.65 | 3.80 | 3.25 | 3.64 | 3.10 | 2.23 | 1.89 |

  The natural frequency follows fn = (1/2π)·sqrt(g sinφ / (L1·A2/A1 + l2)), so it rises with slope angle. — [Krvavica et al., Table 2 and Eq. 33](https://arxiv.org/abs/1903.09314)
- **Response by sea state:** "The strongest air pressure and water level response is found near resonance frequencies."
  - For Hs = 0.4 m (fp 0.34 Hz), the highest response came from pipes with fr 0.263–0.323 Hz. For Hs = 1.0 m (fp 0.22 Hz), it came from the segment with fr = 0.215 Hz.
  - Case 1: max air-pressure amplitudes were in pipes 3 and 5, and minimum in pipes 1 and 6. Water level amplitudes were lowest in segment 1 and highest in the last two.
  - Case 2: pressure was highest in pipe 4 and smallest in the last two.
  - Mean pressure amplitudes reached about 1.2–1.4 kPa (Case 1) and about 3.5–4 kPa (Case 2). Internal water-level amplitudes were about 15–30 cm and about 50–70 cm respectively. These amplitudes are read from Fig. 9 axes and are approximate.
  
  — [Krvavica et al.](https://arxiv.org/abs/1903.09314)
- **Author field observation:** "The sound from the first segment is quieter than the others or even non-existing during small waves, but for higher wave heights, sound from this segment can be heard quite loudly." — [Krvavica et al.](https://arxiv.org/abs/1903.09314)
- **Effect of geometry and waves:** Under identical waves, internal oscillations differ between segments mainly because of pipe geometry. The sound depends "not only on pipe geometry, but also on the length and direction of waves". — [Krvavica et al., Građevinar 2018 (hrcak 201353)](https://hrcak.srce.hr/en/201353)
- **Visitor reports:** when waves are calm "the sound is almost inaudible". Calm days give slower, subtler music, and choppy water gives louder, more intense sound as more pipes activate. Bigger waves make sounds "deeper". These are travel and review sources, so they are qualitative only. — [Wanderlog reviews](https://wanderlog.com/place/details/10236/海のオルガン); [Sail Croatia](https://sail-croatia.com/set-sail/the-zadar-sea-organ); [Absolute Croatia](https://www.absolute-croatia.com/zadar/attractions/sea-organ)

### Inferences
- **"Deeper with bigger waves"** is physically consistent with the model. Larger, longer jugo waves resonate segment 1, whose low natural frequency gives it the longest water column. Segment 1 is an odd (G-major) segment, so its low tones become more present. The pitches are fixed by the acoustic pipes; only which pipes speak changes.
- **Per-segment weighting function:** a usable model weights each segment by a resonance curve, gain_k(f) ∝ 1/sqrt((1 − (f/fn_k)²)² + (2ζ_k f/fn_k)²). The damping is nonlinear (quadratic β|Q|Q) and amplitude-dependent, so ζ is roughly 0.2–0.5 (inference; not reported). Combined with the spectrum S(f), this gives each segment's expected activity for a given sea state.
- **Staggered onsets along the structure (inference, [GENERAL] wave kinematics):**
  - Wave phase speed in deep water is c = gT/2π: about 4.7 m/s at T = 3 s and about 7 m/s at T = 4.5 s. Depth is not known; if water is shallower than L/2 (L ≈ 14 m at 3 s), c is slightly lower.
  - A crest travelling obliquely at angle θ to the wall normal sweeps the about 70 m length in Δt = 70·sinθ / c.
    - θ = 30°: about 7.5 s at 3 s waves, about 5 s at 4.5 s.
    - θ = 60°: about 13 s and about 9 s.
    - Normal incidence (θ ≈ 0°): near-simultaneous onsets.
  - Jugo (SE) and maestral (NW) waves run along the channel axis, so they likely arrive at fairly oblique angles. Expect "rolling" chords travelling from one end of the organ to the other over several seconds. Direction reverses between maestral and jugo. Wake packets from passing vessels also arrive obliquely and sweep the structure.
  - Krvavica et al. state that wave direction influences the sound but give no numbers.
- **Activity regimes for a generative model:**
  - Calm: sparse, isolated notes, mainly segments 2–5, with long gaps of tens of seconds to minutes and bursts during wakes.
  - Maestral: steady, mid-register chordal activity every 2–4 s, grouped into sets every 15–25 s.
  - Jugo: dense, loud, all segments including segment 1, with lower tones prominent, sets every 25–40 s, and near-continuous sound.

### Gaps
- No field acoustic recording analysis was found, such as loudness time series versus measured waves, note-onset statistics, or inter-segment lag measurements.
- The exact orientation of the organ wall, the dominant incident wave directions, and the water depth in front of the organ were not found.
- Krvavica et al. state that the exact dimensions are not public; their Table 1 values are estimates from design drawings.
- No source was found that documents organ behaviour during bura specifically.
