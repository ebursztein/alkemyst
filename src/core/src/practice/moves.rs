use super::{
    Error, PracticeCard, PracticeZone,
    state::{Placed, State},
};
impl State {
    pub fn play(
        &mut self,
        card: u16,
        face: u8,
        destination: PracticeZone,
        cards: &[PracticeCard],
    ) -> Result<(), Error> {
        if !matches!(
            destination,
            PracticeZone::Battlefield | PracticeZone::Graveyard
        ) {
            return Err(Error::Invalid(
                "manual play goes to battlefield or graveyard",
            ));
        }
        let view = cards
            .get(usize::from(card))
            .ok_or(Error::Invalid("unknown physical card"))?;
        if usize::from(face) >= view.faces.len().max(1) {
            return Err(Error::Invalid("unknown card face"));
        }
        let position = self
            .hand
            .iter()
            .position(|id| *id == card)
            .ok_or(Error::Invalid("card is not in hand"))?;
        self.hand.remove(position);
        match destination {
            PracticeZone::Battlefield => self.battlefield.push(Placed {
                card,
                face,
                tapped: false,
            }),
            _ => self.graveyard.push(card),
        }
        Ok(())
    }
    pub fn move_card(&mut self, card: u16, destination: PracticeZone) -> Result<(), Error> {
        let mut found = false;
        for zone in [&mut self.hand, &mut self.graveyard, &mut self.exile] {
            if let Some(i) = zone.iter().position(|id| *id == card) {
                zone.remove(i);
                found = true;
                break;
            }
        }
        if !found && let Some(i) = self.battlefield.iter().position(|p| p.card == card) {
            self.battlefield.remove(i);
            found = true;
        }
        if !found {
            return Err(Error::Invalid("only visible cards may be moved"));
        }
        match destination {
            PracticeZone::Hand => self.hand.push(card),
            PracticeZone::Graveyard => self.graveyard.push(card),
            PracticeZone::Exile => self.exile.push(card),
            PracticeZone::Battlefield => self.battlefield.push(Placed {
                card,
                face: 0,
                tapped: false,
            }),
        }
        Ok(())
    }
}
